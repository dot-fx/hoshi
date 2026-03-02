use std::collections::HashMap;
use rquickjs::{async_with, AsyncContext, AsyncRuntime, CatchResultExt, Function};
use rquickjs::context::EvalOptions;
use rquickjs::function::Opt;
use serde_json::Value;
use crate::error::{CoreError, CoreResult};
use crate::extensions::{ANIME, BASE, BOORU, MANGA, NOVEL, SANDBOX_BOOTSTRAP};

pub(crate) async fn execute_in_quickjs(
    extension_code: String,
    function_name: String,
    args: Vec<Value>,
) -> CoreResult<Value> {
    let base_classes = format!("{}\n{}\n{}\n{}\n{}", BASE, ANIME, MANGA, NOVEL, BOORU);
    let args_json = serde_json::to_string(&args)
        .map_err(|e| CoreError::Internal(format!("Failed to serialize args: {}", e)))?;

    let full_script = build_sandbox_script(
        &base_classes,
        &extension_code,
        &function_name,
        &args_json,
    );

    let json_str = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| CoreError::Internal(format!("Failed to build RT: {}", e)))?;

        let local = tokio::task::LocalSet::new();

        local.block_on(&rt, async move {
            run_quickjs_local(full_script).await
        })
    })
        .await
        .map_err(|e| CoreError::Internal(format!("spawn_blocking panicked: {}", e)))??;

    serde_json::from_str::<Value>(&json_str)
        .map_err(|e| CoreError::Internal(format!("Bad JSON from sandbox: {}\nRaw: {}", e, json_str)))
}

async fn run_quickjs_local(full_script: String) -> CoreResult<String> {
    let rt = AsyncRuntime::new()
        .map_err(|e| CoreError::Internal(format!("QuickJS runtime error: {}", e)))?;

    rt.set_memory_limit(32 * 1024 * 1024).await;
    rt.set_max_stack_size(512 * 1024).await;

    let ctx = AsyncContext::full(&rt)
        .await
        .map_err(|e| CoreError::Internal(format!("QuickJS context error: {}", e)))?;

    let result: Result<String, String> = async_with!(ctx => |ctx| {
        register_native_apis(&ctx)
            .catch(&ctx)
            .map_err(|e| e.to_string())?;

        let opts = EvalOptions::default();

        let val = ctx
            .eval_with_options::<rquickjs::Value, _>(full_script.as_bytes(), opts)
            .catch(&ctx)
            .map_err(|e| e.to_string())?;

        let resolved = if val.is_promise() {
            val.into_promise()
                .unwrap()
                .into_future::<rquickjs::Value>()
                .await
                .catch(&ctx)
                .map_err(|e| e.to_string())?
        } else {
            val
        };

        let json_str = match ctx.json_stringify(resolved).catch(&ctx).map_err(|e| e.to_string())? {
            Some(s) => s.to_string().map_err(|e| e.to_string())?,
            None    => "null".to_string(),
        };

        Ok::<String, String>(json_str)
    })
        .await;

    result.map_err(CoreError::BadRequest)
}

fn build_sandbox_script(
    base_classes: &str,
    extension_code: &str,
    function_name: &str,
    args_json: &str,
) -> String {
    let ext_code_repr = serde_json::to_string(extension_code).unwrap_or_default();

    format!(
        r#"
// ── Bootstrap (polyfills, fetch wrapper, console) ─────
{bootstrap}

// ── Base classes ──────────────────────────────────────
{base}

// ── Runner ────────────────────────────────────────────
(async () => {{
    const VALID_BASES = ["Base", "Anime", "Manga", "Novel", "Booru"];

    const src = {ext_repr};
    const match = src.match(/class\s+([a-zA-Z0-9_]+)\s+extends\s+([a-zA-Z0-9_]+)/);
    if (!match) {{
        throw new Error("No class extending a base was found in the extension");
    }}

    const [, className, parentName] = match;
    if (!VALID_BASES.includes(parentName)) {{
        throw new Error(`Class must extend one of: ${{VALID_BASES.join(", ")}}. Got: ${{parentName}}`);
    }}

    // Carga la clase pasando las base classes como argumentos para que extends funcione
    const ExtClass = new Function("Base", "Anime", "Manga", "Novel", "Booru", `
${{src}}
return ${{className}};
`)(Base, Anime, Manga, Novel, Booru);

    if (typeof ExtClass !== "function") {{
        throw new Error(`Class '${{className}}' could not be loaded`);
    }}

    const instance = new ExtClass();
    if (typeof instance["{fn}"] !== "function") {{
        throw new Error(`Method "{fn}" does not exist on ${{className}}`);
    }}

    const args = {args};
    return await instance["{fn}"](...args);
}})()
"#,
        bootstrap = SANDBOX_BOOTSTRAP,
        base      = base_classes,
        ext_repr  = ext_code_repr,
        fn        = function_name,
        args      = args_json,
    )
}

fn register_native_apis(ctx: &rquickjs::Ctx<'_>) -> rquickjs::Result<()> {
    let globals = ctx.globals();

    let log_fn = Function::new(ctx.clone(), |msg: String| {
        tracing::debug!("{}", msg);  // sin target: usa el crate por defecto
        Ok::<(), rquickjs::Error>(())
    })?;
    globals.set("__native_log", log_fn)?;

    let fetch_fn = Function::new(
        ctx.clone(),
        |url: String, method: String, headers_json: String, body: String| {
            let headers: HashMap<String, String> =
                serde_json::from_str(&headers_json).unwrap_or_default();
            let body_str = if body.is_empty() { None } else { Some(body) };
            let json_result = std::thread::spawn(move || -> String {
                let client = match reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .user_agent("Mozilla/5.0 (compatible; ExtensionSandbox/1.0)")
                    .build()
                {
                    Ok(c)  => c,
                    Err(e) => return error_json(e.to_string()),
                };

                let mut req = match method.to_uppercase().as_str() {
                    "POST"   => client.post(&url),
                    "PUT"    => client.put(&url),
                    "DELETE" => client.delete(&url),
                    "PATCH"  => client.patch(&url),
                    _        => client.get(&url),
                };

                for (k, v) in &headers {
                    req = req.header(k.as_str(), v.as_str());
                }

                if let Some(b) = body_str {
                    req = req.body(b);
                }

                match req.send() {
                    Err(e) => error_json(e.to_string()),
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let ok     = resp.status().is_success();
                        match resp.text() {
                            Err(e) => error_json(e.to_string()),
                            Ok(text) => serde_json::json!({
                                "ok":     ok,
                                "status": status,
                                "body":   text,
                            })
                                .to_string(),
                        }
                    }
                }
            })
                .join()
                .unwrap_or_else(|_| error_json("fetch thread panicked".to_string()));

            Ok::<String, rquickjs::Error>(json_result)
        },
    )?;

    globals.set("__native_fetch", fetch_fn)?;
    Ok(())
}

#[inline]
fn error_json(msg: String) -> String {
    serde_json::json!({ "error": msg }).to_string()
}