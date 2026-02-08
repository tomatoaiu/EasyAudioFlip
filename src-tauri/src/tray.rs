use serde::Serialize;
use std::sync::Mutex;
use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::WebviewWindowBuilder;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, State, WebviewUrl};

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

fn setup_panel_event(app: &AppHandle, panel: &tauri::WebviewWindow) {
    let app_handle = app.clone();
    panel.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if let Some(w) = app_handle.get_webview_window("device-panel") {
                let _ = w.hide();
            }
        }
    });
}

fn create_panel(app: &AppHandle) -> Result<tauri::WebviewWindow, tauri::Error> {
    WebviewWindowBuilder::new(app, "device-panel", WebviewUrl::App("panel.html".into()))
        .title("EasyAudioFlip")
        .inner_size(280.0, 200.0)
        .decorations(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .resizable(false)
        .visible(false)
        .build()
}

fn show_popup_panel(app: &AppHandle) {
    // If panel already exists, toggle visibility
    if let Some(panel) = app.get_webview_window("device-panel") {
        if panel.is_visible().unwrap_or(false) {
            let _ = panel.hide();
        } else {
            reposition_and_show(app, &panel);
        }
        return;
    }

    // Panel doesn't exist yet — create it in a separate thread
    // to avoid WebView2 deadlock on Windows
    let app_handle = app.clone();
    std::thread::spawn(move || {
        match create_panel(&app_handle) {
            Ok(panel) => {
                setup_panel_event(&app_handle, &panel);
                reposition_and_show(&app_handle, &panel);
            }
            Err(e) => {
                eprintln!("Failed to create panel: {}", e);
            }
        }
    });
}

fn reposition_and_show(app: &AppHandle, panel: &tauri::WebviewWindow) {
    // Calculate panel height from device count
    let device_count = {
        let state = app.state::<TrayState>();
        let s = state.app_state.lock().unwrap();
        s.all_devices.len()
    };
    // Each row 32px + hr(9px) + autostart row(36px) + hr(9px) + Quit(32px) + body padding(8px) + buffer(2px)
    let panel_height: f64 = (device_count as f64 * 32.0 + 96.0).max(120.0);
    let panel_width: f64 = 280.0;

    let _ = panel.set_size(LogicalSize::new(panel_width, panel_height));

    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(Some(rect)) = tray.rect() {
            let scale = panel.scale_factor().unwrap_or(1.0);

            // Convert Physical coordinates to Logical via scale_factor
            let (tray_x, tray_y) = match rect.position {
                tauri::Position::Physical(p) => (p.x as f64 / scale, p.y as f64 / scale),
                tauri::Position::Logical(l) => (l.x, l.y),
            };
            let tray_w = match rect.size {
                tauri::Size::Physical(p) => p.width as f64 / scale,
                tauri::Size::Logical(l) => l.width,
            };

            let mut x = tray_x + tray_w - panel_width;
            let mut y = tray_y - panel_height;

            // Clamp to monitor work_area so panel never overlaps the taskbar
            if let Ok(Some(monitor)) = panel.current_monitor() {
                let wa = monitor.work_area();
                let wa_x = wa.position.x as f64 / scale;
                let wa_y = wa.position.y as f64 / scale;
                let wa_w = wa.size.width as f64 / scale;
                let wa_h = wa.size.height as f64 / scale;

                if x < wa_x {
                    x = wa_x;
                }
                if x + panel_width > wa_x + wa_w {
                    x = wa_x + wa_w - panel_width;
                }
                if y < wa_y {
                    y = wa_y;
                }
                if y + panel_height > wa_y + wa_h {
                    y = wa_y + wa_h - panel_height;
                }
            }

            let _ = panel.set_position(LogicalPosition::new(x, y));
        }
    }

    let _ = panel.show();
    let _ = panel.set_focus();
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

    // Tray icon first (critical)
    let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
        .expect("Failed to load tray icon");

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
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

    // Panel window (non-fatal — will be created on first right-click if this fails)
    match create_panel(app.handle()) {
        Ok(panel) => {
            setup_panel_event(app.handle(), &panel);
        }
        Err(e) => {
            eprintln!("Panel pre-creation failed (will retry on right-click): {}", e);
        }
    }

    Ok(())
}
