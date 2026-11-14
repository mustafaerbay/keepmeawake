use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

pub const DON: &str = "don";
const MARGIN: i32 = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Where Don should stand. A saved spot is kept only if Don's centre is on
/// some monitor; otherwise (first launch, unplugged monitor) he goes to the
/// bottom-right of the primary monitor's work area, clear of Dock/taskbar.
pub fn clamp_position(saved: Option<(i32, i32)>, size: (u32, u32), monitors: &[Rect], primary: Rect) -> (i32, i32) {
    let (w, h) = (size.0 as i32, size.1 as i32);
    if let Some((x, y)) = saved {
        let (cx, cy) = (x + w / 2, y + h / 2);
        if monitors.iter().any(|m| cx >= m.x && cx < m.x + m.w && cy >= m.y && cy < m.y + m.h) {
            return (x, y);
        }
    }
    (primary.x + primary.w - w - MARGIN, primary.y + primary.h - h - MARGIN)
}

fn work_rect(m: &tauri::Monitor) -> Rect {
    let a = m.work_area();
    Rect { x: a.position.x, y: a.position.y, w: a.size.width as i32, h: a.size.height as i32 }
}

/// Put Don where he was (or somewhere sane) and return that spot.
pub fn place(app: &AppHandle, saved: Option<(i32, i32)>) -> Option<(i32, i32)> {
    let w = don(app)?;
    let size = w.outer_size().ok()?;
    let monitors: Vec<Rect> = w.available_monitors().ok()?.iter().map(work_rect).collect();
    let primary = work_rect(&w.primary_monitor().ok()??);
    let p = clamp_position(saved, (size.width, size.height), &monitors, primary);
    w.set_position(PhysicalPosition::new(p.0, p.1)).ok()?;
    Some(p)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMARY: Rect = Rect { x: 0, y: 25, w: 1440, h: 800 }; // work area (menu bar + Dock removed)
    const SECOND: Rect = Rect { x: 1440, y: 0, w: 1920, h: 1080 };
    const SIZE: (u32, u32) = (200, 260);

    #[test]
    fn keeps_onscreen_position() {
        assert_eq!(clamp_position(Some((100, 100)), SIZE, &[PRIMARY, SECOND], PRIMARY), (100, 100));
        assert_eq!(clamp_position(Some((2000, 500)), SIZE, &[PRIMARY, SECOND], PRIMARY), (2000, 500));
    }

    #[test]
    fn unplugged_monitor_parks_bottom_right_of_primary_work_area() {
        // saved on the external display, which is gone now
        assert_eq!(clamp_position(Some((3600, 200)), SIZE, &[PRIMARY], PRIMARY), (1440 - 200 - 24, 25 + 800 - 260 - 24));
    }

    #[test]
    fn mostly_offscreen_counts_as_lost() {
        assert_eq!(clamp_position(Some((1420, 100)), SIZE, &[PRIMARY], PRIMARY), (1216, 541));
    }

    #[test]
    fn first_launch_parks_bottom_right() {
        assert_eq!(clamp_position(None, SIZE, &[PRIMARY], PRIMARY), (1216, 541));
    }
}
