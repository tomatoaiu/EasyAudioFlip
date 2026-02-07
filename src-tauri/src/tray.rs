use std::sync::Mutex;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::audio::{self, AppState};
use crate::config::AppConfig;

pub struct TrayState {
    pub app_state: Mutex<AppState>,
}

fn build_tray_menu(
    app: &AppHandle,
    state: &AppState,
) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    let mut builder = MenuBuilder::new(app);

    for device in &state.all_devices {
        let is_enabled = state.enabled_device_ids.contains(&device.id);
        let is_current = state.current_device_id.as_ref() == Some(&device.id);

        let label = if is_current {
            format!("\u{25B6} {}", device.name)
        } else {
            device.name.clone()
        };

        let check_item = CheckMenuItemBuilder::with_id(&device.id, &label)
            .checked(is_enabled)
            .build(app)?;

        builder = builder.item(&check_item);
    }

    builder = builder.separator();

    let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    builder = builder.item(&quit_item);

    builder.build()
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
            let new_menu = build_tray_menu(app, &state).ok();
            drop(state);

            if let Some(tray) = app.tray_by_id("main-tray") {
                let _ = tray.set_tooltip(Some(&tooltip));
                if let Some(menu) = new_menu {
                    let _ = tray.set_menu(Some(menu));
                }
            }
        }
        Ok(None) => {}
        Err(e) => {
            eprintln!("Failed to toggle device: {}", e);
        }
    }
}

fn handle_device_toggle(app: &AppHandle, device_id: &str) {
    let tray_state = app.state::<TrayState>();
    let mut state = tray_state.app_state.lock().unwrap();

    if state.enabled_device_ids.contains(device_id) {
        state.enabled_device_ids.remove(device_id);
    } else {
        state.enabled_device_ids.insert(device_id.to_string());
    }

    let config = AppConfig {
        enabled_device_ids: state.enabled_device_ids.clone(),
    };
    let _ = config.save(app);

    if let Ok(new_menu) = build_tray_menu(app, &state) {
        drop(state);
        if let Some(tray) = app.tray_by_id("main-tray") {
            let _ = tray.set_menu(Some(new_menu));
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
    let menu = build_tray_menu(app.handle(), &app_state)?;

    app.manage(TrayState {
        app_state: Mutex::new(app_state),
    });

    TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip(&tooltip)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                handle_left_click(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => {
                app.exit(0);
            }
            device_id => {
                handle_device_toggle(app, device_id);
            }
        })
        .build(app)?;

    Ok(())
}
