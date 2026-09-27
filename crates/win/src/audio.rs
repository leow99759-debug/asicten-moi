//! System volume, mute and default output device via Core Audio (SPEC §4.4).

use std::ffi::c_void;

use windows::core::{GUID, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCommunications, eConsole, eMultimedia, eRender, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
};

fn err(e: windows::core::Error) -> String {
    e.message()
}

fn enumerator() -> Result<IMMDeviceEnumerator, String> {
    crate::keep_mta();
    // SAFETY: COM init per thread (idempotent; S_FALSE if already initialised).
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).map_err(err)
    }
}

fn endpoint_volume() -> Result<IAudioEndpointVolume, String> {
    let e = enumerator()?;
    // SAFETY: COM calls on valid interfaces.
    unsafe {
        let dev = e.GetDefaultAudioEndpoint(eRender, eConsole).map_err(err)?;
        dev.Activate(CLSCTX_ALL, None).map_err(err)
    }
}

/// Current volume 0–100.
pub fn volume() -> Result<f32, String> {
    // SAFETY: valid interface.
    unsafe { endpoint_volume()?.GetMasterVolumeLevelScalar() }
        .map(|v| v * 100.0)
        .map_err(err)
}

pub fn set_volume(percent: f64) -> Result<(), String> {
    let v = (percent.clamp(0.0, 100.0) / 100.0) as f32;
    // SAFETY: valid interface; null event context.
    unsafe { endpoint_volume()?.SetMasterVolumeLevelScalar(v, std::ptr::null()) }.map_err(err)
}

pub fn change_volume(delta: f64) -> Result<(), String> {
    set_volume(f64::from(volume()?) + delta)
}

pub fn set_mute(mute: bool) -> Result<(), String> {
    // SAFETY: valid interface; null event context.
    unsafe { endpoint_volume()?.SetMute(mute, std::ptr::null()) }.map_err(err)
}

fn friendly_name(dev: &IMMDevice) -> Option<String> {
    // SAFETY: property store read of a valid device; string freed with CoTaskMemFree.
    unsafe {
        let store = dev.OpenPropertyStore(STGM_READ).ok()?;
        let pv = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
        let p = PropVariantToStringAlloc(&pv).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const c_void));
        s
    }
}

/// Names of active output devices.
pub fn output_devices() -> Result<Vec<(String, String)>, String> {
    let e = enumerator()?;
    let mut out = Vec::new();
    // SAFETY: COM enumeration over valid interfaces; ids freed with CoTaskMemFree.
    unsafe {
        let col = e
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(err)?;
        for i in 0..col.GetCount().map_err(err)? {
            let dev = col.Item(i).map_err(err)?;
            let id_p = dev.GetId().map_err(err)?;
            let id = id_p.to_string().unwrap_or_default();
            CoTaskMemFree(Some(id_p.0 as *const c_void));
            if let Some(name) = friendly_name(&dev) {
                out.push((id, name));
            }
        }
    }
    Ok(out)
}

// Undocumented but stable since Windows 7: the only way to change the default device.
// Vtable order from the Windows SDK-era IPolicyConfig; only SetDefaultEndpoint is used.
mod policy {
    #![allow(non_snake_case)]
    use std::ffi::c_void;

    use windows::core::{interface, IUnknown, IUnknown_Vtbl, HRESULT, PCWSTR};

    #[interface("f8679f50-850a-41cf-9c72-430f290290c8")]
    pub unsafe trait IPolicyConfig: IUnknown {
        fn GetMixFormat(&self, a: PCWSTR, b: *mut *mut c_void) -> HRESULT;
        fn GetDeviceFormat(&self, a: PCWSTR, b: i32, c: *mut *mut c_void) -> HRESULT;
        fn ResetDeviceFormat(&self, a: PCWSTR) -> HRESULT;
        fn SetDeviceFormat(&self, a: PCWSTR, b: *mut c_void, c: *mut c_void) -> HRESULT;
        fn GetProcessingPeriod(&self, a: PCWSTR, b: i32, c: *mut i64, d: *mut i64) -> HRESULT;
        fn SetProcessingPeriod(&self, a: PCWSTR, b: *mut i64) -> HRESULT;
        fn GetShareMode(&self, a: PCWSTR, b: *mut c_void) -> HRESULT;
        fn SetShareMode(&self, a: PCWSTR, b: *mut c_void) -> HRESULT;
        fn GetPropertyValue(&self, a: PCWSTR, b: *const c_void, c: *mut c_void) -> HRESULT;
        fn SetPropertyValue(&self, a: PCWSTR, b: *const c_void, c: *mut c_void) -> HRESULT;
        fn SetDefaultEndpoint(&self, id: PCWSTR, role: i32) -> HRESULT;
        fn SetEndpointVisibility(&self, id: PCWSTR, visible: i32) -> HRESULT;
    }

    /// # Safety
    /// `id` must point to a NUL-terminated UTF-16 string; COM must be initialised.
    pub unsafe fn set_default(pc: &IPolicyConfig, id: PCWSTR, role: i32) -> HRESULT {
        // SAFETY: forwarded caller contract.
        unsafe { pc.SetDefaultEndpoint(id, role) }
    }
}
use policy::{set_default, IPolicyConfig};

const CLSID_POLICY_CONFIG: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

/// Make the best-matching output device (by name) the default for all roles.
pub fn switch_device(name: &str) -> Result<(), String> {
    let want = name.to_lowercase();
    let devices = output_devices()?;
    let (id, found) = devices
        .iter()
        .map(|(id, n)| {
            let l = n.to_lowercase();
            let score = if l.contains(&want) {
                2.0
            } else {
                strsim::jaro_winkler(&l, &want)
            };
            (score, id, n)
        })
        .filter(|(s, _, _)| *s >= 0.8)
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id, n)| (id.clone(), n.clone()))
        .ok_or_else(|| format!("устройство «{name}» не найдено"))?;
    let wide: Vec<u16> = id.encode_utf16().chain([0]).collect();
    // SAFETY: COM object created and used on this thread; `wide` outlives the calls.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let pc: IPolicyConfig =
            CoCreateInstance(&CLSID_POLICY_CONFIG, None, CLSCTX_ALL).map_err(err)?;
        for role in [eConsole, eMultimedia, eCommunications] {
            set_default(&pc, PCWSTR(wide.as_ptr()), role.0)
                .ok()
                .map_err(err)?;
        }
    }
    tracing::info!(device = %found, "default output switched");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn reading_volume_does_not_crash() {
        // CI runners may have no audio device; both outcomes are fine, panics are not.
        let _ = super::volume();
        let _ = super::output_devices();
    }
}
