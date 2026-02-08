#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
mod tray;

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::AutoLaunchManager;

/// Max crash log size before rotation (1 MB).
const CRASH_LOG_MAX_BYTES: u64 = 1_000_000;

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

        // Rotate: if crash.log exceeds the size limit, rename it to crash.log.old
        if let Ok(meta) = fs::metadata(&log_path) {
            if meta.len() >= CRASH_LOG_MAX_BYTES {
                let old_path = log_dir.join("crash.log.old");
                let _ = fs::rename(&log_path, &old_path);
            }
        }

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

#[tauri::command]
fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    app.state::<AutoLaunchManager>()
        .is_enabled()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn enable_autostart(app: AppHandle) -> Result<(), String> {
    app.state::<AutoLaunchManager>()
        .enable()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn disable_autostart(app: AppHandle) -> Result<(), String> {
    app.state::<AutoLaunchManager>()
        .disable()
        .map_err(|e| e.to_string())
}

fn main() {
    setup_crash_log();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tray::get_panel_devices,
            tray::toggle_panel_device,
            tray::quit_app,
            is_autostart_enabled,
            enable_autostart,
            disable_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running EasyAudioFlip");
}
