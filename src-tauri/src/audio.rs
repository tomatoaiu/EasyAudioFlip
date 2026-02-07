use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub all_devices: Vec<AudioDevice>,
    pub enabled_device_ids: HashSet<String>,
    pub current_device_id: Option<String>,
}

#[cfg(windows)]
mod platform {
    use super::AudioDevice;
    use windows::core::PCWSTR;
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{
        eConsole, eCommunications, eMultimedia, eRender, IMMDeviceEnumerator,
        MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
    };
    use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;

    pub fn init_com() -> windows::core::Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()
        }
    }

    pub fn enumerate_devices() -> windows::core::Result<Vec<AudioDevice>> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

            let collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            let count = collection.GetCount()?;

            let mut devices = Vec::with_capacity(count as usize);
            for i in 0..count {
                let device = collection.Item(i)?;

                let id_pwstr = device.GetId()?;
                let id = id_pwstr.to_string()?;
                windows::Win32::System::Com::CoTaskMemFree(Some(id_pwstr.0 as *const _));

                let store = device.OpenPropertyStore(STGM_READ)?;
                let prop = store.GetValue(&PKEY_Device_FriendlyName)?;
                let name_pwstr = PropVariantToStringAlloc(&prop)?;
                let name = name_pwstr.to_string()?;
                windows::Win32::System::Com::CoTaskMemFree(Some(name_pwstr.0 as *const _));

                devices.push(AudioDevice { id, name });
            }
            Ok(devices)
        }
    }

    pub fn get_default_device_id() -> windows::core::Result<String> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let id_pwstr = device.GetId()?;
            let id = id_pwstr.to_string()?;
            windows::Win32::System::Com::CoTaskMemFree(Some(id_pwstr.0 as *const _));
            Ok(id)
        }
    }

    pub fn set_default_device(device_id: &str) -> windows::core::Result<()> {
        use com_policy_config::{IPolicyConfig, PolicyConfigClient};

        unsafe {
            let policy_config: IPolicyConfig =
                CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;

            let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr = PCWSTR(wide.as_ptr());

            policy_config.SetDefaultEndpoint(pcwstr, eConsole)?;
            policy_config.SetDefaultEndpoint(pcwstr, eMultimedia)?;
            policy_config.SetDefaultEndpoint(pcwstr, eCommunications)?;
            Ok(())
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::AudioDevice;

    pub fn init_com() -> Result<(), String> {
        Ok(())
    }

    pub fn enumerate_devices() -> Result<Vec<AudioDevice>, String> {
        Ok(vec![
            AudioDevice {
                id: "stub-speaker".to_string(),
                name: "Speakers (Stub)".to_string(),
            },
            AudioDevice {
                id: "stub-headphone".to_string(),
                name: "Headphones (Stub)".to_string(),
            },
        ])
    }

    pub fn get_default_device_id() -> Result<String, String> {
        Ok("stub-speaker".to_string())
    }

    pub fn set_default_device(_device_id: &str) -> Result<(), String> {
        Ok(())
    }
}

pub use platform::*;

pub fn toggle_next_device(state: &mut AppState) -> Result<Option<AudioDevice>, String> {
    let rotation: Vec<&AudioDevice> = state
        .all_devices
        .iter()
        .filter(|d| state.enabled_device_ids.contains(&d.id))
        .collect();

    if rotation.len() < 2 {
        return Ok(None);
    }

    let current_index = state
        .current_device_id
        .as_ref()
        .and_then(|cid| rotation.iter().position(|d| &d.id == cid))
        .unwrap_or(0);

    let next_index = (current_index + 1) % rotation.len();
    let next_device = rotation[next_index].clone();

    set_default_device(&next_device.id).map_err(|e| format!("{:?}", e))?;
    state.current_device_id = Some(next_device.id.clone());

    Ok(Some(next_device))
}
