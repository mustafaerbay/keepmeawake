use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

pub const DON: &str = "don";

#[derive(Clone, serde::Serialize)]
struct Visibility {
    visible: bool,
}

pub fn don(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(DON)
}

/// Floating, on every Space and over fullscreen apps.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let Some(w) = don(app) else { return Ok(()) };
    w.set_visible_on_all_workspaces(true)?;
    #[cfg(target_os = "macos")]
    mac_join_all_spaces(&w);
    Ok(())
}

#[cfg(target_os = "macos")]
fn mac_join_all_spaces(w: &WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    const CAN_JOIN_ALL_SPACES: usize = 1 << 0;
    const STATIONARY: usize = 1 << 4;
    const FULL_SCREEN_AUXILIARY: usize = 1 << 8;
    if let Ok(ptr) = w.ns_window() {
        let ns = ptr as *mut AnyObject;
        unsafe {
            let cur: usize = msg_send![ns, collectionBehavior];
            let _: () = msg_send![ns, setCollectionBehavior: cur | CAN_JOIN_ALL_SPACES | STATIONARY | FULL_SCREEN_AUXILIARY];
        }
    }
}

pub fn is_don_visible(app: &AppHandle) -> bool {
    don(app).and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

pub fn set_don_visible(app: &AppHandle, visible: bool) {
    if let Some(w) = don(app) {
        let _ = if visible { w.show() } else { w.hide() };
    }
    let _ = app.emit("don-visibility", Visibility { visible });
}

pub fn set_click_through(app: &AppHandle, on: bool) {
    if let Some(w) = don(app) {
        let _ = w.set_ignore_cursor_events(on);
    }
}
