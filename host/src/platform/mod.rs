//! Platform seam: every Windows-vs-Unix difference lives behind these four
//! functions, so the rest of the host stays platform-neutral.

use std::path::PathBuf;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod imp;
#[cfg(not(target_os = "windows"))]
#[path = "unix.rs"]
mod imp;

/// Resolve a libimobiledevice CLI (`iproxy`, `idevice_id`, `ideviceinfo`) to a
/// runnable path. `None` means it is not installed / not found.
pub fn tool_path(name: &str) -> Option<PathBuf> {
    imp::tool_path(name)
}

/// Per-OS cache directory, created if missing.
pub fn cache_dir() -> Option<PathBuf> {
    imp::cache_dir()
}

/// Human OS string for the `--debug` header.
pub fn os_version() -> String {
    imp::os_version()
}

/// One actionable line on how to install the USB tools on this OS.
pub fn missing_tools_hint() -> &'static str {
    imp::missing_tools_hint()
}

/// Return the first `dirs` entry that contains a file named `name{exe_suffix}`.
/// Platform-neutral so it is unit-testable; the OS layers supply `exe_suffix`
/// (".exe" on Windows, "" elsewhere) and the directory list.
#[allow(dead_code)]
pub(crate) fn resolve_in_dirs(
    name: &str,
    exe_suffix: &str,
    dirs: impl IntoIterator<Item = PathBuf>,
) -> Option<PathBuf> {
    let filename = format!("{name}{exe_suffix}");
    for dir in dirs {
        let cand = dir.join(&filename);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_tmp() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("ioscpy-test-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn resolves_first_dir_that_has_the_file() {
        let d1 = unique_tmp();
        let d2 = unique_tmp();
        let target = d2.join("iproxy.exe");
        fs::write(&target, b"x").unwrap();

        let got = resolve_in_dirs("iproxy", ".exe", vec![d1.clone(), d2.clone()]);
        assert_eq!(got, Some(target));

        let none = resolve_in_dirs("nope", ".exe", vec![d1, d2]);
        assert_eq!(none, None);
    }
}
