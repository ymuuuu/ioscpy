//! macOS + Linux implementation of the platform seam. This reproduces the
//! behavior the host had before the seam existed: bare tool names resolved by
//! `Command` via PATH, the per-OS cache dir (macOS `~/Library/Caches/ioscpy`,
//! Linux `$XDG_CACHE_HOME`/`~/.cache`), `sw_vers`/`uname`, and the existing
//! install hints.

use std::path::PathBuf;
use std::process::Command;

/// Return the bare name; `Command` resolves it through PATH, exactly as before.
/// Always `Some`, so the spawn site keeps its original "fails at exec" behavior.
pub fn tool_path(name: &str) -> Option<PathBuf> {
    Some(PathBuf::from(name))
}

pub fn cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")?;
        let mut p = PathBuf::from(home);
        p.push("Library/Caches/ioscpy");
        let _ = std::fs::create_dir_all(&p);
        Some(p)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let mut p = if let Some(x) = std::env::var_os("XDG_CACHE_HOME") {
            PathBuf::from(x)
        } else {
            let home = std::env::var_os("HOME")?;
            let mut h = PathBuf::from(home);
            h.push(".cache");
            h
        };
        p.push("ioscpy");
        let _ = std::fs::create_dir_all(&p);
        Some(p)
    }
}

pub fn os_version() -> String {
    #[cfg(target_os = "macos")]
    let (cmd, args): (&str, &[&str]) = ("sw_vers", &["-productVersion"]);
    #[cfg(not(target_os = "macos"))]
    let (cmd, args): (&str, &[&str]) = ("uname", &["-sr"]);

    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn missing_tools_hint() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "The USB tools are missing. Install them with:  brew install libimobiledevice"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "The USB tools are missing. Install them with:  sudo apt install libimobiledevice-utils usbmuxd"
    }
}
