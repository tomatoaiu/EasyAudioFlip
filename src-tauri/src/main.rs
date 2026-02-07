#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
mod tray;

fn setup_crash_log() {
    use std::fs;
    use std::io::Write;
    use std::panic;

    panic::set_hook(Box::new(|info| {
        let log_dir = dirs::config_dir()
            .unwrap_or_default()
            .join("com.easyaudioflip.desktop");
        let _ = fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("crash.log");
        let msg = format!(
            "[{}] PANIC: {}\nLocation: {:?}\n\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            info,
            info.location(),
        );
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut f| f.write_all(msg.as_bytes()));
        eprintln!("{}", msg);
    }));
}

fn main() {
    setup_crash_log();

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
