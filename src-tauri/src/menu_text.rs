//! Exact menu copy from the spec, kept in one place.

pub const DISPLAY: &str = "Ekran kararabilir";
pub const CLICK_THROUGH: &str = "Don'a dokunulmasın";
pub const THEME: &str = "Tema";
pub const TRAY_ANIM: &str = "Hareketli ikon";
pub const SIZE: &str = "Boyut";

pub fn size_label(s: crate::window::DonSize) -> &'static str {
    use crate::window::DonSize::*;
    match s {
        Small => "Küçük",
        Medium => "Orta",
        Large => "Büyük",
    }
}
pub const AUTOSTART: &str = "Açılışta başlat";
pub const QUIT: &str = "Çıkış";
pub const COFFEE: &str = "☕ Don'a kahve ısmarla";
pub const COFFEE_URL: &str = "https://buymeacoffee.com/mustafaerbay";
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
    fn size_labels() {
        use crate::window::DonSize::*;
        assert_eq!(SIZE, "Boyut");
        assert_eq!(size_label(Small), "Küçük");
        assert_eq!(size_label(Medium), "Orta");
        assert_eq!(size_label(Large), "Büyük");
    }

    #[test]
    fn coffee_item() {
        assert_eq!(COFFEE, "☕ Don'a kahve ısmarla");
        assert_eq!(COFFEE_URL, "https://buymeacoffee.com/mustafaerbay");
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
