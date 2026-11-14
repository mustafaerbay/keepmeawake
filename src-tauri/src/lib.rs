pub mod controller;
pub mod menu_text;
pub mod power;
pub mod settings;
pub mod tray;
pub mod tray_frames;
pub mod window;

use controller::{Controller, State};
use settings::Settings;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub type AppCtl = Mutex<Controller<power::PlatformPower>>;
pub struct ThemeId(pub Mutex<String>);

pub const DEFAULT_THEME: &str = "don-tombul";

#[derive(Clone, serde::Serialize)]
struct StatePayload<'a> {
    state: State,
    transition: Option<&'a str>,
}

pub fn emit_state(app: &AppHandle, s: State, transition: Option<&str>) {
    let _ = app.emit("state-changed", StatePayload { state: s, transition });
}

/// Every awake toggle, from Don or from the tray, goes through here.
pub fn toggle(app: &AppHandle) -> State {
    let s = app.state::<AppCtl>().lock().unwrap().toggle();
    emit_state(app, s, Some(if s.awake { "lighting" } else { "snuffing" }));
    tray::sync(app);
    s
}

#[tauri::command]
fn get_state(ctl: tauri::State<AppCtl>) -> State {
    ctl.lock().unwrap().state()
}

#[tauri::command]
fn toggle_awake(app: AppHandle) -> State {
    toggle(&app)
}

#[tauri::command]
fn get_theme(theme: tauri::State<ThemeId>) -> String {
    theme.0.lock().unwrap().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_notification::init())
        // Managed up front with defaults so the window can never ask before
        // they exist; setup() then applies the saved settings.
        .manage::<AppCtl>(Mutex::new(Controller::new(power::platform(), false, false)))
        .manage(ThemeId(Mutex::new(DEFAULT_THEME.into())))
        .manage(Mutex::new(Settings::default()))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let h = app.handle();
            let mut saved = settings::load(h);
            {
                let ctl = h.state::<AppCtl>();
                let mut c = ctl.lock().unwrap();
                c.set_display_may_sleep(saved.display_may_sleep);
                c.set_click_through(saved.click_through);
            }
            *h.state::<ThemeId>().0.lock().unwrap() = saved.theme.clone();
            let _ = h.emit("theme-changed", serde_json::json!({ "id": saved.theme }));

            window::setup(h)?;
            window::apply_size(h, saved.don_size, false);
            saved.don_pos = window::place(h, saved.don_pos).or(saved.don_pos);
            window::set_click_through(h, saved.click_through);
            window::set_don_visible(h, saved.don_visible);
            if let Some(w) = window::don(h) {
                let h2 = h.clone();
                w.on_window_event(move |e| {
                    if let tauri::WindowEvent::Moved(p) = e {
                        h2.state::<Mutex<Settings>>().lock().unwrap().don_pos = Some((p.x, p.y));
                    }
                });
            }
            *h.state::<Mutex<Settings>>().lock().unwrap() = saved;
            tray::build(h)?;

            // After the tray exists, so the icon the hint talks about is there.
            #[cfg(windows)]
            {
                use tauri_plugin_notification::NotificationExt;
                let st = h.state::<Mutex<Settings>>();
                let mut s = st.lock().unwrap();
                settings::show_hint_once(&mut s, || {
                    h.notification().builder().title("keepmeawake").body(menu_text::WIN_HINT).show().map_err(|e| e.to_string())
                });
                settings::save(h, &s);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_state, toggle_awake, get_theme])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, e| {
            if let tauri::RunEvent::Exit = e {
                // Release explicitly instead of relying on process teardown,
                // and keep where Don was dragged to.
                app.state::<AppCtl>().lock().unwrap().set_awake(false);
                let s = app.state::<Mutex<Settings>>().lock().unwrap().clone();
                settings::save(app, &s);
            }
        });
}
