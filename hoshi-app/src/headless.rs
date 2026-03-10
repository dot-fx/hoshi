//! TauriHeadless — implementación del trait HeadlessBrowser para Tauri v2.
//!
//! ## Mecanismo de retorno de valores
//!
//! `WebviewWindow::eval()` ejecuta JS pero no devuelve el valor. La solución
//! portable (funciona en WKWebView, WebView2, WebKitGTK, Android WebView) es:
//!
//!   1. El init script, cuando la página está lista, llama:
//!        `window.__TAURI_INTERNALS__.postMessage({ cmd: 'headless_done', payload })`
//!   2. Rust escucha con `webview.once("headless-done", callback)` — esto
//!        es el sistema de eventos nativos de Tauri, disponible en todas las plataformas.
//!   3. Un contador atómico global genera labels únicos (hl-1, hl-2...) sin UUID.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::oneshot;

use hoshi_core::error::{CoreError, CoreResult};
use hoshi_core::headless::{
    BlockedResource, Cookie, CapturedRequest, HeadlessBrowser,
    HeadlessOptions, HeadlessResponse, WaitFor,
};

// ---------------------------------------------------------------------------
// Label único sin UUID — contador atómico global
// ---------------------------------------------------------------------------

static HEADLESS_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_label() -> String {
    format!("hl-{}", HEADLESS_COUNTER.fetch_add(1, Ordering::Relaxed))
}

// ---------------------------------------------------------------------------
// Payload que el JS envía de vuelta a Rust vía el evento "headless-done"
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HeadlessDonePayload {
    label:    String,
    url:      String,
    html:     String,
    result:   Option<serde_json::Value>,
    captured: Vec<CapturedRequest>,
}

// ---------------------------------------------------------------------------
// TauriHeadless
// ---------------------------------------------------------------------------

pub struct TauriHeadless<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriHeadless<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl<R: Runtime + 'static> HeadlessBrowser for TauriHeadless<R> {
    fn is_available(&self) -> bool { true }

    async fn fetch(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        let label      = next_label();
        let url        = url.to_string();
        let timeout_ms = options.timeout_ms;
        let app        = self.app.clone();
        let label_c    = label.clone();
        let options_c  = options.clone();

        // Canal para recibir el resultado del listener
        let (tx, rx) = oneshot::channel::<CoreResult<HeadlessDonePayload>>();
        let tx       = Arc::new(std::sync::Mutex::new(Some(tx)));

        // Registrar el listener ANTES de crear el webview para no perder el evento
        {
            let tx_l    = tx.clone();
            let label_l = label.clone();
            let app_l   = app.clone();

            // once() escucha un solo evento — perfecto para one-shot
            app.once(format!("headless-done-{}", label), move |event| {
                let result = serde_json::from_str::<HeadlessDonePayload>(event.payload())
                    .map_err(|e| CoreError::Internal(format!("headless payload parse: {}", e)));

                // Limpiar el webview
                if let Some(w) = app_l.get_webview_window(&label_l) {
                    let _ = w.destroy();
                }

                if let Ok(mut guard) = tx_l.lock() {
                    if let Some(sender) = guard.take() {
                        let _ = sender.send(result);
                    }
                }
            });
        }

        // Crear el webview headless en el main thread
        let app_create = app.clone();
        app.run_on_main_thread(move || {
            if let Err(e) = create_headless_webview(&app_create, &label_c, &url, &options_c) {
                tracing::error!("[headless] failed to create webview '{}': {}", label_c, e);
                // Notificar el error si el webview no se pudo crear
                // (el listener de arriba no recibirá nada, llegará al timeout)
            }
        }).map_err(|e| CoreError::Internal(format!("run_on_main_thread: {:?}", e)))?;

        // Esperar resultado con timeout
        match tokio::time::timeout(Duration::from_millis(timeout_ms), rx).await {
            Ok(Ok(Ok(payload))) => {
                Ok(HeadlessResponse {
                    url:      payload.url,
                    status:   200, // WebView no expone el HTTP status
                    html:     payload.html,
                    result:   payload.result,
                    captured: payload.captured,
                    cookies:  vec![], // ver nota abajo
                })
            }
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(_))     => Err(CoreError::Internal("headless channel dropped".into())),
            Err(_)         => {
                // Timeout — limpiar el webview
                let label_t  = label.clone();
                let app_t    = app.clone();
                let app_t2   = app_t.clone();
                let _ = app_t.run_on_main_thread(move || {
                    if let Some(w) = app_t2.get_webview_window(&label_t) {
                        let _ = w.destroy();
                    }
                });
                Err(CoreError::Internal(
                    format!("headless timeout after {}ms", timeout_ms)
                ))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Crear el webview oculto
// ---------------------------------------------------------------------------

fn create_headless_webview<R: Runtime>(
    app: &AppHandle<R>,
    label: &str,
    url: &str,
    options: &HeadlessOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_url: url::Url = url.parse()?;
    let init_script           = build_init_script(label, options);
    let blocked_patterns      = build_blocked_url_patterns(&options.block);

    let mut builder = WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::External(parsed_url),
    )
        .visible(false)
        .decorations(false)
        .inner_size(1280.0, 800.0)
        .initialization_script(&init_script);

    if !blocked_patterns.is_empty() {
        builder = builder.on_navigation(move |nav_url| {
            let s = nav_url.as_str();
            !blocked_patterns.iter().any(|p| s.contains(p.as_str()))
        });
    }

    builder.build()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Init script — se inyecta antes de que cargue la página
// ---------------------------------------------------------------------------

fn build_init_script(label: &str, options: &HeadlessOptions) -> String {
    let label_json        = serde_json::to_string(label).unwrap_or_default();
    let capture_json      = serde_json::to_string(&options.capture).unwrap_or_default();
    let js_to_eval_json   = serde_json::to_string(
        &options.javascript.clone().unwrap_or_default()
    ).unwrap_or_default();
    let wait_is_idle      = matches!(options.wait_for, WaitFor::NetworkIdle);
    let wait_selector     = match &options.wait_for {
        WaitFor::Selector(s) => s.clone(),
        _                    => String::new(),
    };
    let wait_selector_json = serde_json::to_string(&wait_selector).unwrap_or_default();
    let block_css         = build_css_block_rules(&options.block);

    format!(r#"
(function() {{
    "use strict";

    // ── Bloqueo CSS de recursos pesados ──────────────────────────────────────
    {block_css}

    // ── Interceptor fetch/XHR para capturar requests ─────────────────────────
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

    // ── Enviar resultado a Rust via evento Tauri ──────────────────────────────
    async function __hoshi_collect() {{
        const label = {label_json};

        let userResult = null;
        const jsCode = {js_to_eval_json};
        if (jsCode) {{
            try {{ userResult = eval(jsCode); }} catch(e) {{ userResult = {{ error: e.message }}; }}
            // Si el resultado es una Promise, resolverla
            if (userResult && typeof userResult.then === 'function') {{
                try {{ userResult = await userResult; }} catch(e) {{ userResult = {{ error: e.message }}; }}
            }}
        }}

        const payload = {{
            label,
            url:      window.location.href,
            html:     document.documentElement.outerHTML,
            result:   userResult,
            captured: __captured,
        }};

        // Emitir evento hacia Rust via Tauri v2
        // Enviamos el objeto directamente (sin JSON.stringify extra)
        // para que Rust lo reciba como JSON parseable en event.payload()
        const eventName = 'headless-done-' + label;
        try {{
            await window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {{
                event: eventName,
                payload: payload,
            }});
        }} catch(e) {{
            try {{
                await window.__TAURI__.event.emit(eventName, payload);
            }} catch(e2) {{
                console.error('[hoshi] failed to emit headless event:', e.message, e2 && e2.message);
            }}
        }}
    }}

    // ── Esperar según wait_for ────────────────────────────────────────────────
    const __waitIsIdle     = {wait_is_idle};
    const __waitSelector   = {wait_selector_json};

    function __hoshi_wait_and_collect() {{
        if (__waitSelector) {{
            // Esperar a que aparezca el selector con MutationObserver
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
            // Fallback timeout por si el selector nunca aparece
            setTimeout(function() {{ obs.disconnect(); __hoshi_collect(); }}, 8000);

        }} else if (__waitIsIdle) {{
            // NetworkIdle: 600ms sin actividad de red
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
            // DomReady — ya estamos aquí
            __hoshi_collect();
        }}
    }}

    if (document.readyState === 'complete' || document.readyState === 'interactive') {{
        __hoshi_wait_and_collect();
    }} else {{
        window.addEventListener('DOMContentLoaded', __hoshi_wait_and_collect, {{ once: true }});
    }}

}})();
"#,
            block_css          = block_css,
            capture_json       = capture_json,
            label_json         = label_json,
            js_to_eval_json    = js_to_eval_json,
            wait_is_idle       = wait_is_idle,
            wait_selector_json = wait_selector_json,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_css_block_rules(blocked: &[BlockedResource]) -> String {
    let mut rules = vec![];
    for r in blocked {
        match r {
            BlockedResource::Images     => rules.push("img,picture,image{display:none!important}"),
            BlockedResource::Fonts      => rules.push("@font-face{src:local('')!important}"),
            BlockedResource::Media      => rules.push("video,audio{display:none!important}"),
            BlockedResource::Stylesheet |
            BlockedResource::Pattern(_) => {}
        }
    }
    if rules.is_empty() { return String::new(); }
    format!(
        r#"(function(){{const s=document.createElement('style');s.textContent=`{}`;(document.head||document.documentElement).appendChild(s);}})();"#,
        rules.join("")
    )
}

fn build_blocked_url_patterns(blocked: &[BlockedResource]) -> Vec<String> {
    let mut patterns = vec![
        // Ad networks y trackers siempre bloqueados
        "googlesyndication.com".into(),
        "doubleclick.net".into(),
        "adservice.google".into(),
        "googletagmanager.com".into(),
        "google-analytics.com".into(),
        "facebook.com/tr".into(),
        "amazon-adsystem.com".into(),
        "scorecardresearch.com".into(),
    ];
    for r in blocked {
        match r {
            BlockedResource::Pattern(p) => patterns.push(p.clone()),
            BlockedResource::Fonts => {
                patterns.extend([".woff2".into(), ".woff".into(), ".ttf".into(), ".otf".into()]);
            }
            BlockedResource::Media => {
                patterns.extend([".mp4".into(), ".webm".into(), ".mp3".into(), ".ogg".into()]);
            }
            BlockedResource::Stylesheet => {
                patterns.push(".css".into());
            }
            BlockedResource::Images => {
                // Las imágenes son navegación válida en muchos sitios,
                // mejor ocultarlas via CSS que bloquearlas via URL
            }
        }
    }
    patterns
}