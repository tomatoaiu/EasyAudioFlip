#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
mod tray;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tray::get_panel_devices,
            tray::toggle_panel_device,
            tray::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running EasyAudioFlip");
}
