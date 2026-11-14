pub mod controller;
pub mod power;
pub mod window;

use controller::{Controller, State};
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
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION)
                .build(),
        )
        .manage::<AppCtl>(Mutex::new(Controller::new(power::platform(), false, false)))
        .manage(ThemeId(Mutex::new(DEFAULT_THEME.into())))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            window::setup(app.handle())?;
            window::set_don_visible(app.handle(), true);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_state, toggle_awake, get_theme])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
