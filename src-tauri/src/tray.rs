use serde::Serialize;
use std::sync::Mutex;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, LogicalPosition, Manager, State, WebviewUrl};

use crate::audio::{self, AppState};
use crate::config::AppConfig;

pub struct TrayState {
    pub app_state: Mutex<AppState>,
}

#[derive(Clone, Serialize)]
pub struct PanelDevice {
    id: String,
    name: String,
    enabled: bool,
    is_current: bool,
}

fn devices_from_state(state: &AppState) -> Vec<PanelDevice> {
    state
        .all_devices
        .iter()
        .map(|d| PanelDevice {
            id: d.id.clone(),
            name: d.name.clone(),
            enabled: state.enabled_device_ids.contains(&d.id),
            is_current: state.current_device_id.as_ref() == Some(&d.id),
        })
        .collect()
}

#[tauri::command]
pub fn get_panel_devices(state: State<TrayState>) -> Vec<PanelDevice> {
    let s = state.app_state.lock().unwrap();
    devices_from_state(&s)
}

#[tauri::command]
pub fn toggle_panel_device(
    state: State<TrayState>,
    app: AppHandle,
    device_id: String,
) -> Vec<PanelDevice> {
    let mut s = state.app_state.lock().unwrap();

    if s.enabled_device_ids.contains(&device_id) {
        s.enabled_device_ids.remove(&device_id);
    } else {
        s.enabled_device_ids.insert(device_id);
    }

    let config = AppConfig {
        enabled_device_ids: s.enabled_device_ids.clone(),
    };
    let _ = config.save(&app);

    devices_from_state(&s)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

fn get_tooltip_text(state: &AppState) -> String {
    match &state.current_device_id {
        Some(id) => {
            let name = state
                .all_devices
                .iter()
                .find(|d| &d.id == id)
                .map(|d| d.name.as_str())
                .unwrap_or("Unknown");
            format!("EasyAudioFlip - {}", name)
        }
        None => "EasyAudioFlip".to_string(),
    }
}

fn handle_left_click(app: &AppHandle) {
    let tray_state = app.state::<TrayState>();
    let mut state = tray_state.app_state.lock().unwrap();

    match audio::toggle_next_device(&mut state) {
        Ok(Some(new_device)) => {
            let tooltip = format!("EasyAudioFlip - {}", new_device.name);
            drop(state);

            if let Some(tray) = app.tray_by_id("main-tray") {
                let _ = tray.set_tooltip(Some(&tooltip));
            }
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("Failed to toggle device: {}", e);
        }
    }
}

fn show_popup_panel(app: &AppHandle) {
    // Toggle: close existing panel if open
    if let Some(existing) = app.get_webview_window("device-panel") {
        let _ = existing.close();
        return;
    }

    let device_count = {
        let tray_state = app.state::<TrayState>();
        let state = tray_state.app_state.lock().unwrap();
        state.all_devices.len()
    };

    let panel_width: f64 = 280.0;
    let panel_height: f64 = (device_count as f64) * 36.0 + 52.0;

    // Calculate position near tray icon
    let position = if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(Some(rect)) = tray.rect() {
            let (tray_x, tray_y) = match rect.position {
                tauri::Position::Physical(p) => (p.x as f64, p.y as f64),
                tauri::Position::Logical(l) => (l.x, l.y),
            };
            let tray_w = match rect.size {
                tauri::Size::Physical(p) => p.width as f64,
                tauri::Size::Logical(l) => l.width,
            };
            // Position above the tray icon, aligned to its right edge
            let x = tray_x - panel_width + tray_w;
            let y = tray_y - panel_height;
            LogicalPosition::new(x, y)
        } else {
            LogicalPosition::new(100.0, 100.0)
        }
    } else {
        LogicalPosition::new(100.0, 100.0)
    };

    let builder = WebviewWindowBuilder::new(
        app,
        "device-panel",
        WebviewUrl::App("panel.html".into()),
    )
    .title("EasyAudioFlip")
    .inner_size(panel_width, panel_height)
    .position(position.x, position.y)
    .decorations(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .focused(true)
    .visible(true)
    .resizable(false);

    match builder.build() {
        Ok(window) => {
            let app_handle = app.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    if let Some(w) = app_handle.get_webview_window("device-panel") {
                        let _ = w.close();
                    }
                }
            });
        }
        Err(e) => {
            eprintln!("Failed to create panel window: {}", e);
        }
    }
}

pub fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    audio::init_com().expect("Failed to initialize COM");

    let config = AppConfig::load(app.handle());

    let all_devices = audio::enumerate_devices().unwrap_or_default();

    let current_device_id = audio::get_default_device_id().ok();

    let enabled_ids = if config.enabled_device_ids.is_empty() {
        all_devices.iter().map(|d| d.id.clone()).collect()
    } else {
        config.enabled_device_ids
    };

    let app_state = AppState {
        all_devices,
        enabled_device_ids: enabled_ids,
        current_device_id,
    };

    let tooltip = get_tooltip_text(&app_state);

    app.manage(TrayState {
        app_state: Mutex::new(app_state),
    });

    TrayIconBuilder::with_id("main-tray")
        .tooltip(&tooltip)
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                handle_left_click(tray.app_handle());
            }
            TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                ..
            } => {
                show_popup_panel(tray.app_handle());
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
