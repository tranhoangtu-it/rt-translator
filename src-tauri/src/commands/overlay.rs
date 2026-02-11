use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Open the caption overlay window. If already open, show it.
#[tauri::command]
pub async fn open_overlay_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, "overlay", WebviewUrl::App("overlay.html".into()))
        .title("Caption Overlay")
        .inner_size(600.0, 180.0)
        .transparent(true)
        .decorations(false)
        .always_on_top(true)
        .resizable(true)
        .visible(true)
        .build()
        .map_err(|e| e.to_string())?;

    tracing::info!("Overlay window opened");
    Ok(())
}

/// Close the caption overlay window.
#[tauri::command]
pub async fn close_overlay_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        window.close().map_err(|e| e.to_string())?;
        tracing::info!("Overlay window closed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Overlay commands require a running Tauri app context,
    // so they are tested via integration tests / manual testing.
}
