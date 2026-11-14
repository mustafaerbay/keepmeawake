//! Exact menu copy from the spec, kept in one place.

pub const DISPLAY: &str = "Ekran kararabilir";
pub const CLICK_THROUGH: &str = "Don'a dokunulmasın";
pub const THEME: &str = "Tema";
pub const TRAY_ANIM: &str = "Hareketli ikon";
pub const AUTOSTART: &str = "Açılışta başlat";
pub const QUIT: &str = "Çıkış";
pub const WIN_HINT: &str = "Don'u görev çubuğuna sabitlemek için ikonu sürükle";

pub fn toggle_label(awake: bool) -> &'static str {
    if awake { "💨 Puroyu söndür" } else { "🔥 Puroyu yak" }
}

pub fn don_label(visible: bool) -> &'static str {
    if visible { "Don'u gizle" } else { "Don'u göster" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_label_follows_awake() {
        assert_eq!(toggle_label(true), "💨 Puroyu söndür");
        assert_eq!(toggle_label(false), "🔥 Puroyu yak");
    }

    #[test]
    fn don_label_follows_visibility() {
        assert_eq!(don_label(true), "Don'u gizle");
        assert_eq!(don_label(false), "Don'u göster");
    }

    #[test]
    fn fixed_labels() {
        assert_eq!(DISPLAY, "Ekran kararabilir");
        assert_eq!(CLICK_THROUGH, "Don'a dokunulmasın");
        assert_eq!(THEME, "Tema");
        assert_eq!(TRAY_ANIM, "Hareketli ikon");
        assert_eq!(AUTOSTART, "Açılışta başlat");
        assert_eq!(QUIT, "Çıkış");
        assert_eq!(WIN_HINT, "Don'u görev çubuğuna sabitlemek için ikonu sürükle");
    }
}
