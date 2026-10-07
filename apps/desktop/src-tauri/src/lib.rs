pub mod connection;
pub mod input;
pub mod protocol;
pub mod server;
pub mod webrtc_manager;
pub mod signaling_client;

use std::sync::Arc;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use std::net::UdpSocket;

#[tauri::command]
fn get_local_ip() -> String {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect("8.8.8.8:80").unwrap();
    socket.local_addr().unwrap().ip().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_local_ip])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Phase 6: macOS Polish (System Tray)
            let quit_i = MenuItem::with_id(app, "quit", "Quit MovaPad", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            // Mouse controller and server initialization
            let mouse = input::create_mouse_controller();
            let mouse_arc: Arc<dyn input::MouseController> = mouse.into();
            let mouse_clone = Arc::clone(&mouse_arc);
            
            tauri::async_runtime::spawn(async move {
                if let Err(e) = server::start_local_server("0.0.0.0:8081", mouse_clone, 1.5).await {
                    log::error!("Local server failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
