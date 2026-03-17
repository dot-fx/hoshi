use std::time::Duration;
use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;

use hoshi_core::error::{CoreError, CoreResult};
use hoshi_core::headless::{
    BlockedResource, CapturedRequest, HeadlessBrowser,
    HeadlessOptions, HeadlessResponse, WaitFor,
};

pub struct AxumHeadless {
    browser: Browser,
}

impl AxumHeadless {

    pub async fn new() -> anyhow::Result<Self> {
        let config = BrowserConfig::builder()
            .window_size(1920, 1080)
            .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--accept-lang=es-ES,es;q=0.9,en;q=0.8")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-extensions")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build Chromium config: {}", e))?;

        let (browser, mut handler) = Browser::launch(config).await
            .map_err(|e| anyhow::anyhow!("Failed to init Chromium: {}", e))?;

        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    continue;
                }
            }
        });

        Ok(Self { browser })
    }

    async fn execute_fetch(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        let page = self.browser.new_page("about:blank").await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let init_script = build_evaluate_on_new_document_script(&options);
        page.evaluate_on_new_document(init_script.as_str()).await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        page.goto(url).await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let mut is_done = false;
        for _ in 0..150 { // Máximo 15 segundos
            let check = page.evaluate("window.__hoshi_done === true").await;
            if let Ok(res) = check {
                if res.into_value::<bool>().unwrap_or(false) {
                    is_done = true;
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        if !is_done {
            let _ = page.close().await;
            return Err(CoreError::Internal("Timeout esperando a que finalice el script inyectado (¿selector no encontrado?)".to_string()));
        }

        let payload_json: serde_json::Value = page
            .evaluate("window.__hoshi_payload || {}")
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?
            .into_value()
            .unwrap_or(serde_json::json!({}));

        let _ = page.close().await;

        let html = payload_json.get("html").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let result = payload_json.get("result").cloned();
        let captured: Vec<CapturedRequest> = payload_json.get("captured")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(HeadlessResponse {
            url: url.to_string(),
            status: 200,
            html,
            result,
            captured,
            cookies: vec![],
        })
    }
}

#[async_trait]
impl HeadlessBrowser for AxumHeadless {
    fn is_available(&self) -> bool {
        true
    }

    async fn fetch(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        let timeout_ms = options.timeout_ms;

        match tokio::time::timeout(Duration::from_millis(timeout_ms), self.execute_fetch(url, options)).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(CoreError::Internal(format!("headless timeout after {}ms", timeout_ms))),
        }
    }
}

fn build_evaluate_on_new_document_script(options: &HeadlessOptions) -> String {
    let capture_json = serde_json::to_string(&options.capture).unwrap_or_default();
    let js_to_eval_json = serde_json::to_string(&options.javascript.clone().unwrap_or_default()).unwrap_or_default();
    let wait_is_idle = matches!(options.wait_for, WaitFor::NetworkIdle);
    let wait_selector = match &options.wait_for {
        WaitFor::Selector(s) => s.clone(),
        _ => String::new(),
    };
    let wait_selector_json = serde_json::to_string(&wait_selector).unwrap_or_default();
    let block_css = build_css_block_rules(&options.block);

    format!(r#"
    (function() {{
        "use strict";

        // ── EVASIÓN ANTI-BOT (Stealth) ──────────────────────────────────────────
        // Ocultar que somos un entorno automatizado
        Object.defineProperty(navigator, 'webdriver', {{ get: () => undefined }});

        // Simular que el navegador tiene interfaz (Cloudflare busca esto)
        window.chrome = {{ runtime: {{}} }};

        // Simular plugins (en headless suele estar vacío)
        Object.defineProperty(navigator, 'plugins', {{ get: () => [1, 2, 3] }});
        Object.defineProperty(navigator, 'languages', {{ get: () => ['es-ES', 'es', 'en', 'en-US'] }});
        // ────────────────────────────────────────────────────────────────────────

        window.__hoshi_done = false;
        window.__hoshi_payload = {{}};

        window.__hoshi_done = false;
        window.__hoshi_payload = {{}};

        {block_css}

        const __capturePatterns = {capture_json};
        const __captured = [];

        if (__capturePatterns.length > 0) {{
            const __origFetch = window.fetch ? window.fetch.bind(window) : null;
            if (__origFetch) {{
                window.fetch = async function(input, init) {{
                    const url    = typeof input === 'string' ? input : (input && input.url) || '';
                    const method = (init && init.method) || 'GET';
                    const resp   = await __origFetch(input, init);
                    if (__capturePatterns.some(p => url.includes(p))) {{
                        try {{
                            const clone   = resp.clone();
                            const body    = await clone.text().catch(() => null);
                            const headers = {{}};
                            clone.headers.forEach((v, k) => {{ headers[k] = v; }});
                            __captured.push({{ url, method, status: resp.status, body, headers }});
                        }} catch(_) {{}}
                    }}
                    return resp;
                }};
            }}

            const __OrigXHR = window.XMLHttpRequest;
            window.XMLHttpRequest = function() {{
                const xhr = new __OrigXHR();
                let __xurl = '', __xmethod = '';
                const origOpen = xhr.open.bind(xhr);
                xhr.open = function(m, u, ...r) {{ __xurl = u; __xmethod = m; return origOpen(m, u, ...r); }};
                xhr.addEventListener('load', function() {{
                    if (__capturePatterns.some(p => __xurl.includes(p))) {{
                        __captured.push({{
                            url: __xurl, method: __xmethod,
                            status: xhr.status, body: xhr.responseText, headers: {{}}
                        }});
                    }}
                }});
                return xhr;
            }};
        }}

        async function __hoshi_collect() {{
            if (window.__hoshi_done) return; // Evitar llamadas dobles

            let userResult = null;
            const jsCode = {js_to_eval_json};
            if (jsCode) {{
                try {{ userResult = eval(jsCode); }} catch(e) {{ userResult = {{ error: e.message }}; }}
                if (userResult && typeof userResult.then === 'function') {{
                    try {{ userResult = await userResult; }} catch(e) {{ userResult = {{ error: e.message }}; }}
                }}
            }}

            window.__hoshi_payload = {{
                html:     document.documentElement.outerHTML,
                result:   userResult,
                captured: __captured,
            }};

            // Le decimos a Rust que ya puede extraer los datos
            window.__hoshi_done = true;
        }}

        const __waitIsIdle     = {wait_is_idle};
        const __waitSelector   = {wait_selector_json};

        function __hoshi_wait_and_collect() {{
            if (__waitSelector) {{
                if (document.querySelector(__waitSelector)) {{
                    __hoshi_collect();
                    return;
                }}
                const obs = new MutationObserver(function() {{
                    if (document.querySelector(__waitSelector)) {{
                        obs.disconnect();
                        __hoshi_collect();
                    }}
                }});
                obs.observe(document.documentElement, {{ childList: true, subtree: true }});
                setTimeout(function() {{ obs.disconnect(); __hoshi_collect(); }}, 8000);

            }} else if (__waitIsIdle) {{
                let __idleTimer = setTimeout(__hoshi_collect, 1500);
                const __resetIdle = function() {{
                    clearTimeout(__idleTimer);
                    __idleTimer = setTimeout(__hoshi_collect, 600);
                }};
                const __origFetch2 = window.fetch ? window.fetch.bind(window) : null;
                if (__origFetch2) {{
                    window.fetch = function(...args) {{
                        const p = __origFetch2(...args);
                        __resetIdle();
                        p.finally(__resetIdle);
                        return p;
                    }};
                }}
            }} else {{
                __hoshi_collect();
            }}
        }}

        // Aseguramos que la función de espera se inyecte correctamente
        if (document.readyState === 'complete' || document.readyState === 'interactive') {{
            __hoshi_wait_and_collect();
        }} else {{
            window.addEventListener('DOMContentLoaded', __hoshi_wait_and_collect, {{ once: true }});
        }}

    }})();
    "#,
            block_css = block_css,
            capture_json = capture_json,
            js_to_eval_json = js_to_eval_json,
            wait_is_idle = wait_is_idle,
            wait_selector_json = wait_selector_json,
    )
}

fn build_css_block_rules(blocked: &[BlockedResource]) -> String {
    let mut rules = vec![];
    for r in blocked {
        match r {
            BlockedResource::Images => rules.push("img,picture,image{display:none!important}"),
            BlockedResource::Fonts => rules.push("@font-face{src:local('')!important}"),
            BlockedResource::Media => rules.push("video,audio{display:none!important}"),
            BlockedResource::Stylesheet | BlockedResource::Pattern(_) => {}
        }
    }
    if rules.is_empty() { return String::new(); }
    format!(
        r#"document.addEventListener('DOMContentLoaded', () => {{ const s=document.createElement('style'); s.textContent=`{}`; (document.head||document.documentElement).appendChild(s); }});"#,
        rules.join("")
    )
}