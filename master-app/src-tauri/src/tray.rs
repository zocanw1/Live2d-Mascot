use tauri::{
    menu::{Menu, MenuItem},
    AppHandle, Manager, Runtime, Emitter,
};

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let menu = Menu::with_items(app, &[
        &MenuItem::with_id(app, "show", "Show", true, None::<&str>).map_err(|e| e.to_string())?,
        &MenuItem::with_id(app, "start_overlay", "Start Overlay", true, None::<&str>).map_err(|e| e.to_string())?,
        &MenuItem::with_id(app, "stop_overlay", "Stop Overlay", false, None::<&str>).map_err(|e| e.to_string())?,
        &MenuItem::with_id(app, "separator1", "", false, None::<&str>).map_err(|e| e.to_string())?,
        &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?,
    ]).map_err(|e| e.to_string())?;
    
    tauri::tray::TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event_id: &str) -> Result<(), String> {
    match event_id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "start_overlay" => {
            app.emit("tray-start-overlay", ()).map_err(|e| e.to_string())?;
        }
        "stop_overlay" => {
            app.emit("tray-stop-overlay", ()).map_err(|e| e.to_string())?;
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
    Ok(())
}

pub fn update_tray_menu<R: Runtime>(app: &AppHandle<R>, overlay_running: bool) -> Result<(), String> {
    if let Some(tray) = app.tray_by_id("main") {
        let menu = Menu::with_items(app, &[
            &MenuItem::with_id(app, "show", "Show", true, None::<&str>).map_err(|e| e.to_string())?,
            &MenuItem::with_id(app, "start_overlay", "Start Overlay", !overlay_running, None::<&str>).map_err(|e| e.to_string())?,
            &MenuItem::with_id(app, "stop_overlay", "Stop Overlay", overlay_running, None::<&str>).map_err(|e| e.to_string())?,
            &MenuItem::with_id(app, "separator1", "", false, None::<&str>).map_err(|e| e.to_string())?,
            &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?,
        ]).map_err(|e| e.to_string())?;
        
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
