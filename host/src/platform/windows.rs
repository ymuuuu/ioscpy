//! Windows implementation of the platform seam. Reaches the device through the
//! installed libimobiledevice-win32 CLIs, which talk to Apple's usbmuxd
//! (127.0.0.1:27015) from Apple Mobile Device Support. No USB driver is touched.

use std::path::PathBuf;
use std::process::Command;

/// Resolve a CLI to a full path: search PATH, then the directory of the running
/// `ioscpy.exe` (so a user can drop `iproxy.exe` next to it). `.exe` is appended.
pub fn tool_path(name: &str) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.to_path_buf());
        }
    }
    super::resolve_in_dirs(name, ".exe", dirs)
}

pub fn cache_dir() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    let mut p = PathBuf::from(base);
    p.push("ioscpy");
    let _ = std::fs::create_dir_all(&p);
    Some(p)
}

pub fn os_version() -> String {
    // `cmd /C ver` prints something like "Microsoft Windows [Version 10.0.22631.0]".
    Command::new("cmd")
        .args(["/C", "ver"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Windows".to_string())
}

pub fn missing_tools_hint() -> &'static str {
    "Install Apple Mobile Device Support (from iTunes) and libimobiledevice-win32, \
     then put iproxy.exe, idevice_id.exe, and ideviceinfo.exe on your PATH or next \
     to ioscpy.exe."
}
