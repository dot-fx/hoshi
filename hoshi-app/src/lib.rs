use tauri::{Manager, async_runtime};


pub fn run() -> anyhow::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            async_runtime::block_on(async {
                let state = hoshi_core::build_app_state().await?;
                app.manage(state);
                Ok::<(), anyhow::Error>(())
            })?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Tauri runtime error: {}", e))?;

    Ok(())
}