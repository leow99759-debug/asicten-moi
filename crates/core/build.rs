//! sherpa-onnx prebuilt libs are compiled with a newer MSVC and call STL helpers that
//! MSVC < 14.44 doesn't ship (LNK2001 `__std_find_first_of_trivial_pos_1`). On such a
//! toolset, enable `old_msvc_stl` so `msvc_compat` provides them.

fn main() {
    println!("cargo:rustc-check-cfg=cfg(old_msvc_stl)");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=VCToolsVersion");
    println!("cargo:rerun-if-env-changed=VCINSTALLDIR");
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        return;
    }
    let target = std::env::var("TARGET").unwrap_or_default();
    let Some(tool) = find_msvc_tools::find_tool(&target, "link.exe") else {
        return;
    };
    // …\VC\Tools\MSVC\14.43.34808\bin\HostX64\x64\link.exe
    let Some(dir) = tool
        .path()
        .ancestors()
        .find(|p| p.parent().and_then(|m| m.file_name()) == Some("MSVC".as_ref()))
    else {
        return;
    };
    let Some(ver) = dir.file_name() else { return };
    // Re-run after a Build Tools upgrade, else the shim would clash with the new STL.
    if let Some(vc) = dir.ancestors().nth(3) {
        let default = vc.join(r"Auxiliary\Build\Microsoft.VCToolsVersion.default.txt");
        println!("cargo:rerun-if-changed={}", default.display());
    }
    let ver = ver.to_string_lossy();
    let mut parts = ver.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    let (major, minor) = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
    if (major, minor) < (14, 44) {
        println!("cargo:rustc-cfg=old_msvc_stl");
    }
}
