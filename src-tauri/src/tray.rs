use crate::controller::State;
use crate::settings::Settings;
use crate::tray_frames::{self, Frames};
use crate::window::DonSize;
use crate::{menu_text as txt, window, AppCtl, ThemeId};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_autostart::ManagerExt;

const TRAY_ID: &str = "main";
/// One puff cycle starts this often while lit and "Hareketli ikon" is on.
pub const PUFF_EVERY: Duration = Duration::from_secs(8);
/// How long each puff frame stays up.
pub const PUFF_FRAME: Duration = Duration::from_millis(200);

/// Windows tray icons are 16 px at 100% scale; pick the closest rendered size.
pub fn win_icon_size(scale: f64) -> u32 {
    let want = 16.0 * scale;
    *[16u32, 20, 24, 32]
        .iter()
        .min_by(|a, b| (**a as f64 - want).abs().total_cmp(&(**b as f64 - want).abs()))
        .unwrap()
}

pub fn frames_for(theme: &str, platform: &str, px: u32) -> Option<&'static Frames> {
    tray_frames::table()
        .iter()
        .find(|(id, p, s, _)| *id == theme && *p == platform && *s == px)
        .map(|(_, _, _, f)| f)
}

fn platform_frames(app: &AppHandle, theme: &str) -> &'static Frames {
    let pick = |t: &str| {
        if cfg!(target_os = "macos") {
            frames_for(t, "mac", 36)
        } else {
            let scale = app.primary_monitor().ok().flatten().map(|m| m.scale_factor()).unwrap_or(1.0);
            frames_for(t, "win", win_icon_size(scale))
        }
    };
    pick(theme).or_else(|| pick(crate::DEFAULT_THEME)).expect("default theme frames")
}

struct Items {
    toggle: MenuItem<Wry>,
    don: MenuItem<Wry>,
    display: CheckMenuItem<Wry>,
    click: CheckMenuItem<Wry>,
    anim: CheckMenuItem<Wry>,
    sizes: Vec<(DonSize, CheckMenuItem<Wry>)>,
    themes: Vec<(&'static str, CheckMenuItem<Wry>)>,
    autostart: CheckMenuItem<Wry>,
}

#[derive(Default)]
struct AnimState {
    awake: bool,
    animated: bool,
    theme: String,
}

pub struct TrayUi {
    items: Items,
    anim: Arc<(Mutex<AnimState>, Condvar)>,
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let items = Items {
        toggle: MenuItem::with_id(app, "toggle", txt::toggle_label(true), true, None::<&str>)?,
        don: MenuItem::with_id(app, "don", txt::don_label(true), true, None::<&str>)?,
        display: CheckMenuItem::with_id(app, "display", txt::DISPLAY, true, false, None::<&str>)?,
        click: CheckMenuItem::with_id(app, "click", txt::CLICK_THROUGH, true, false, None::<&str>)?,
        anim: CheckMenuItem::with_id(app, "anim", txt::TRAY_ANIM, true, true, None::<&str>)?,
        sizes: DonSize::ALL
            .iter()
            .map(|sz| {
                CheckMenuItem::with_id(app, format!("size:{}", sz.id()), txt::size_label(*sz), true, false, None::<&str>)
                    .map(|i| (*sz, i))
            })
            .collect::<tauri::Result<_>>()?,
        themes: tray_frames::THEMES
            .iter()
            .map(|(id, name)| {
                CheckMenuItem::with_id(app, format!("theme:{id}"), *name, true, false, None::<&str>).map(|i| (*id, i))
            })
            .collect::<tauri::Result<_>>()?,
        autostart: CheckMenuItem::with_id(app, "autostart", txt::AUTOSTART, true, false, None::<&str>)?,
    };
    let theme_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        items.themes.iter().map(|(_, i)| i as &dyn tauri::menu::IsMenuItem<Wry>).collect();
    let theme_menu = Submenu::with_items(app, txt::THEME, true, &theme_refs)?;
    let size_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        items.sizes.iter().map(|(_, i)| i as &dyn tauri::menu::IsMenuItem<Wry>).collect();
    let size_menu = Submenu::with_items(app, txt::SIZE, true, &size_refs)?;
    let quit = MenuItem::with_id(app, "quit", txt::QUIT, true, None::<&str>)?;
    let sep = || PredefinedMenuItem::separator(app);
    let menu = Menu::with_items(
        app,
        &[&items.toggle, &items.don, &sep()?, &items.display, &items.click, &items.anim, &size_menu, &theme_menu, &items.autostart, &sep()?, &quit],
    )?;

    let theme = app.state::<ThemeId>().0.lock().unwrap().clone();
    let first = Image::from_bytes(platform_frames(app, &theme).awake[0])?;
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(first)
        .icon_as_template(false)
        .tooltip("keepmeawake")
        .menu(&menu)
        // macOS: click opens the menu. Windows: left-click toggles, right-click opens the menu.
        .show_menu_on_left_click(cfg!(target_os = "macos"))
        .on_menu_event(on_menu)
        .on_tray_icon_event(|tray, e| {
            if cfg!(not(target_os = "macos")) {
                if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                    crate::toggle(tray.app_handle());
                }
            }
        })
        .build(app)?;

    let anim = Arc::new((Mutex::new(AnimState::default()), Condvar::new()));
    app.manage(TrayUi { items, anim: anim.clone() });
    spawn_animation(app.clone(), tray, anim);
    sync(app);
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Draw {
    Lit(usize),
    Cold,
}

/// What the tray shows at `step` of the puff cycle, how long until the next
/// step (`None` = until state changes), and the next step. Step 0 is the
/// resting lit frame; steps 1..n are one short puff. A continuously animated
/// icon cost ~2.7% CPU, an occasional puff costs a fraction of that.
pub fn tray_plan(awake: bool, animated: bool, step: usize, n: usize) -> (Draw, Option<Duration>, usize) {
    if !awake {
        return (Draw::Cold, None, 0);
    }
    if !animated || n < 2 {
        return (Draw::Lit(0), None, 0);
    }
    if step == 0 || step >= n {
        return (Draw::Lit(0), Some(PUFF_EVERY - PUFF_FRAME * (n as u32 - 1)), 1);
    }
    let next = if step + 1 >= n { 0 } else { step + 1 };
    (Draw::Lit(step), Some(PUFF_FRAME), next)
}

/// Redraws the tray icon only when what it should show changes; between
/// puffs it sleeps on the condvar.
fn spawn_animation(app: AppHandle, tray: TrayIcon, anim: Arc<(Mutex<AnimState>, Condvar)>) {
    std::thread::spawn(move || {
        let (lock, cv) = &*anim;
        let mut shown: Option<(String, Draw)> = None;
        let mut step = 0usize;
        let mut g = lock.lock().unwrap();
        loop {
            // Theme "" means sync() has not run yet.
            if g.theme.is_empty() {
                g = cv.wait(g).unwrap();
                continue;
            }
            let theme = g.theme.clone();
            let f = platform_frames(&app, &theme);
            let (draw, wait, next) = tray_plan(g.awake, g.animated, step, f.awake.len());
            step = next;
            if shown.as_ref() != Some(&(theme.clone(), draw)) {
                drop(g);
                let bytes = match draw {
                    Draw::Lit(i) => f.awake[i],
                    Draw::Cold => f.asleep,
                };
                if let Ok(img) = Image::from_bytes(bytes) {
                    let _ = tray.set_icon(Some(img));
                }
                shown = Some((theme, draw));
                g = lock.lock().unwrap();
            }
            g = match wait {
                Some(d) => cv.wait_timeout(g, d).unwrap().0,
                None => cv.wait(g).unwrap(),
            };
        }
    });
}

fn on_menu(app: &AppHandle, e: MenuEvent) {
    let id = e.id().as_ref();
    let ctl = app.state::<AppCtl>();
    match id {
        "toggle" => {
            crate::toggle(app);
            return;
        }
        "don" => window::set_don_visible(app, !window::is_don_visible(app)),
        "display" => {
            let cur = ctl.lock().unwrap().state();
            let s = ctl.lock().unwrap().set_display_may_sleep(!cur.display_may_sleep);
            crate::emit_state(app, s, None);
        }
        "click" => {
            let cur = ctl.lock().unwrap().state();
            let s = ctl.lock().unwrap().set_click_through(!cur.click_through);
            window::set_click_through(app, s.click_through);
            crate::emit_state(app, s, None);
        }
        "anim" => {
            let st = app.state::<Mutex<Settings>>();
            let mut cur = st.lock().unwrap();
            cur.tray_anim = !cur.tray_anim;
        }
        "autostart" => {
            let al = app.autolaunch();
            let _ = if al.is_enabled().unwrap_or(false) { al.disable() } else { al.enable() };
        }
        "quit" => {
            app.exit(0);
            return;
        }
        other => {
            if let Some(sz) = DonSize::ALL.iter().find(|sz| other == format!("size:{}", sz.id())) {
                app.state::<Mutex<Settings>>().lock().unwrap().don_size = *sz;
                window::apply_size(app, *sz, true);
            } else if let Some(theme) = other.strip_prefix("theme:") {
                *app.state::<ThemeId>().0.lock().unwrap() = theme.to_string();
                let _ = app.emit("theme-changed", serde_json::json!({ "id": theme }));
            }
        }
    }
    sync(app);
}

/// Make menu, tray animation and the saved settings match current state.
pub fn sync(app: &AppHandle) {
    let s: State = app.state::<AppCtl>().lock().unwrap().state();
    let visible = window::is_don_visible(app);
    let theme = app.state::<ThemeId>().0.lock().unwrap().clone();
    let animated = app.state::<Mutex<Settings>>().lock().unwrap().tray_anim;
    if let Some(ui) = app.try_state::<TrayUi>() {
        let it = &ui.items;
        let _ = it.toggle.set_text(txt::toggle_label(s.awake));
        let _ = it.don.set_text(txt::don_label(visible));
        let _ = it.display.set_checked(s.display_may_sleep);
        let _ = it.click.set_checked(s.click_through);
        let _ = it.anim.set_checked(animated);
        let size = app.state::<Mutex<Settings>>().lock().unwrap().don_size;
        for (sz, item) in &it.sizes {
            let _ = item.set_checked(*sz == size);
        }
        for (id, item) in &it.themes {
            let _ = item.set_checked(*id == theme);
        }
        let _ = it.autostart.set_checked(app.autolaunch().is_enabled().unwrap_or(false));
        let (lock, cv) = &*ui.anim;
        *lock.lock().unwrap() = AnimState { awake: s.awake, animated, theme: theme.clone() };
        cv.notify_all();
    }
    let cur = app.state::<Mutex<Settings>>().lock().unwrap().clone();
    let next = Settings {
        display_may_sleep: s.display_may_sleep,
        click_through: s.click_through,
        don_visible: visible,
        theme,
        ..cur
    };
    *app.state::<Mutex<Settings>>().lock().unwrap() = next.clone();
    crate::settings::save(app, &next);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_size_matches_dpi() {
        assert_eq!(win_icon_size(1.0), 16);
        assert_eq!(win_icon_size(1.25), 20);
        assert_eq!(win_icon_size(1.5), 24);
        assert_eq!(win_icon_size(1.75), 24);
        assert_eq!(win_icon_size(2.0), 32);
        assert_eq!(win_icon_size(3.0), 32);
    }

    #[test]
    fn every_theme_has_frames_for_every_platform_size() {
        for (id, _) in tray_frames::THEMES {
            assert_eq!(frames_for(id, "mac", 36).unwrap().awake.len(), 6);
            for px in [16, 20, 24, 32] {
                assert!(frames_for(id, "win", px).is_some(), "{id} win {px}");
            }
        }
    }

    const N: usize = 6;

    #[test]
    fn asleep_is_cold_and_waits_for_a_change() {
        assert_eq!(tray_plan(false, true, 3, N), (Draw::Cold, None, 0));
        assert_eq!(tray_plan(false, false, 0, N), (Draw::Cold, None, 0));
    }

    #[test]
    fn static_icon_when_animation_off() {
        assert_eq!(tray_plan(true, false, 4, N), (Draw::Lit(0), None, 0));
    }

    #[test]
    fn rests_then_puffs_through_every_frame_then_rests() {
        let (d, w, next) = tray_plan(true, true, 0, N);
        assert_eq!((d, w), (Draw::Lit(0), Some(PUFF_EVERY - PUFF_FRAME * (N as u32 - 1))));
        let mut step = next;
        let mut seen = vec![];
        loop {
            let (d, w, next) = tray_plan(true, true, step, N);
            seen.push(d);
            if next == 0 { assert_eq!(w, Some(PUFF_FRAME)); break; }
            assert_eq!(w, Some(PUFF_FRAME));
            step = next;
        }
        assert_eq!(seen, (1..N).map(Draw::Lit).collect::<Vec<_>>());
    }

    #[test]
    fn puff_is_short_and_rare() {
        // ~1 s of animation every 8 s keeps the tray cheap
        assert_eq!(PUFF_EVERY, Duration::from_secs(8));
        assert!(PUFF_FRAME * (N as u32 - 1) <= Duration::from_millis(1200));
    }

    #[test]
    fn unknown_theme_has_no_frames() {
        assert!(frames_for("nope", "mac", 36).is_none());
    }
}
