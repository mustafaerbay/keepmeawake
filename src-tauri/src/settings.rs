use serde::{Deserialize, Serialize};

pub const STORE_FILE: &str = "settings.json";
pub const STORE_KEY: &str = "settings";

/// Everything that survives a restart. Awake is deliberately absent:
/// the app always starts lit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub display_may_sleep: bool,
    pub click_through: bool,
    pub don_visible: bool,
    pub theme: String,
    pub hint_shown: bool,
    /// Don window top-left in physical pixels.
    pub don_pos: Option<(i32, i32)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            display_may_sleep: false,
            click_through: false,
            don_visible: true,
            theme: crate::DEFAULT_THEME.into(),
            hint_shown: false,
            don_pos: None,
        }
    }
}

impl Settings {
    /// Tolerant: a missing, corrupt or stale store never stops the app.
    pub fn from_json(v: Option<serde_json::Value>) -> Settings {
        let mut s: Settings = v.and_then(|v| serde_json::from_value(v).ok()).unwrap_or_default();
        if !crate::tray_frames::THEMES.iter().any(|(id, _)| *id == s.theme) {
            s.theme = crate::DEFAULT_THEME.into();
        }
        s
    }
}

/// First-run hint: shown at most once, and only counted when it really showed.
#[allow(dead_code)] // used on Windows only
pub fn show_hint_once(s: &mut Settings, show: impl FnOnce() -> Result<(), String>) {
    if !s.hint_shown && show().is_ok() {
        s.hint_shown = true;
    }
}

pub fn load(app: &tauri::AppHandle) -> Settings {
    use tauri_plugin_store::StoreExt;
    Settings::from_json(app.store(STORE_FILE).ok().and_then(|st| st.get(STORE_KEY)))
}

pub fn save(app: &tauri::AppHandle, s: &Settings) {
    use tauri_plugin_store::StoreExt;
    if let (Ok(st), Ok(v)) = (app.store(STORE_FILE), serde_json::to_value(s)) {
        st.set(STORE_KEY, v);
        let _ = st.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn missing_store_gives_defaults() {
        let s = Settings::from_json(None);
        assert_eq!(s, Settings {
            display_may_sleep: false,
            click_through: false,
            don_visible: true,
            theme: "don-tombul".into(),
            hint_shown: false,
            don_pos: None,
        });
    }

    #[test]
    fn partial_store_keeps_known_fields() {
        let s = Settings::from_json(Some(json!({ "theme": "don-sert", "donVisible": false })));
        assert_eq!(s.theme, "don-sert");
        assert!(!s.don_visible);
        assert!(!s.display_may_sleep);
    }

    #[test]
    fn garbage_store_gives_defaults() {
        assert_eq!(Settings::from_json(Some(json!("nope"))), Settings::default());
        assert_eq!(Settings::from_json(Some(json!({ "theme": 42 }))), Settings::default());
    }

    #[test]
    fn unknown_theme_falls_back_to_default() {
        let s = Settings::from_json(Some(json!({ "theme": "don-deleted" })));
        assert_eq!(s.theme, "don-tombul");
    }

    #[test]
    fn round_trips() {
        let s = Settings { display_may_sleep: true, click_through: true, don_visible: false, theme: "don-sinsi".into(), hint_shown: true, don_pos: Some((-1200, 40)) };
        assert_eq!(Settings::from_json(Some(serde_json::to_value(&s).unwrap())), s);
    }

    #[test]
    fn hint_marked_shown_only_when_shown() {
        let mut s = Settings::default();
        show_hint_once(&mut s, || Err("no notifications".into()));
        assert!(!s.hint_shown, "a failed notification must be retried next launch");
        show_hint_once(&mut s, || Ok(()));
        assert!(s.hint_shown);
        let mut called = false;
        show_hint_once(&mut s, || { called = true; Ok(()) });
        assert!(!called, "never shown twice");
    }

    #[test]
    fn awake_is_never_persisted() {
        let v = serde_json::to_value(Settings::default()).unwrap();
        assert!(v.get("awake").is_none());
    }
}
