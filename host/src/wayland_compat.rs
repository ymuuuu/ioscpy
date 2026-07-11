//! Workaround for compositors without server-side decorations (issue #4).
//!
//! minifb's Wayland backend requests decorations via `zxdg_decoration_manager_v1`
//! and has no client-side fallback, so on compositors that refuse server-side
//! decorations (GNOME/mutter) the window comes up with no titlebar, no resize
//! handles and no close button. KDE/KWin and wlroots compositors advertise the
//! protocol and are fine.
//!
//! Instead of asking users to run `WAYLAND_DISPLAY= ioscpy`, we probe the
//! compositor's global registry once at startup: if it does not advertise
//! `zxdg_decoration_manager_v1`, we unset `WAYLAND_DISPLAY` for this process so
//! minifb (and the clipboard) fall back to X11/XWayland, which decorates the
//! window normally. `--wayland` opts out and keeps the native connection.

use wayland_client::{
    protocol::wl_registry::{Event as RegistryEvent, WlRegistry},
    Connection, Dispatch, QueueHandle,
};

const DECORATION_MANAGER: &str = "zxdg_decoration_manager_v1";

struct Probe {
    has_decoration_manager: bool,
}

impl Dispatch<WlRegistry, ()> for Probe {
    fn event(
        state: &mut Self,
        _registry: &WlRegistry,
        event: RegistryEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let RegistryEvent::Global { interface, .. } = event {
            if interface == DECORATION_MANAGER {
                state.has_decoration_manager = true;
            }
        }
    }
}

/// True if the compositor advertises server-side decorations, None if we could
/// not talk to it at all (then we leave the environment alone and let minifb
/// surface whatever error there is).
fn compositor_has_server_decorations() -> Option<bool> {
    let conn = Connection::connect_to_env().ok()?;
    let display = conn.display();
    let mut queue = conn.new_event_queue();
    let qh = queue.handle();
    display.get_registry(&qh, ());
    let mut probe = Probe {
        has_decoration_manager: false,
    };
    // One roundtrip is enough: the registry announces every global before the
    // sync callback fires.
    queue.roundtrip(&mut probe).ok()?;
    Some(probe.has_decoration_manager)
}

fn env_set(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|v| !v.is_empty())
}

/// Call once at startup, before any window or clipboard is created and before
/// any thread is spawned (this may mutate the process environment).
pub fn apply_decoration_workaround(keep_native_wayland: bool) {
    // WAYLAND_SOCKET (an inherited connection fd) takes precedence over
    // WAYLAND_DISPLAY in libwayland and wayland-client alike, so either one
    // means minifb would connect to Wayland.
    if !env_set("WAYLAND_DISPLAY") && !env_set("WAYLAND_SOCKET") {
        return; // not a Wayland session (or already masked by the user)
    }
    if keep_native_wayland {
        return;
    }
    if compositor_has_server_decorations() != Some(false) {
        return;
    }
    std::env::remove_var("WAYLAND_DISPLAY");
    std::env::remove_var("WAYLAND_SOCKET");
    eprintln!(
        "ioscpy: this Wayland compositor draws no window decorations for us \
         (no {DECORATION_MANAGER}); using X11/XWayland instead. \
         Pass --wayland to keep native Wayland."
    );
}
