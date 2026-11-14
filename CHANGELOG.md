# Değişiklik günlüğü

Bu projedeki önemli değişiklikler bu dosyada tutulur.
Biçim [Keep a Changelog](https://keepachangelog.com/tr-TR/1.1.0/) kurallarına uyar, sürüm numaraları [Semantic Versioning](https://semver.org/lang/tr/) ile verilir.

Yeni bir sürüm çıkarmak için:

1. `## [Unreleased]` altındakileri yeni bir `## [X.Y.Z] - YYYY-AA-GG` başlığına taşı.
2. `package.json`, `src-tauri/Cargo.toml` ve `src-tauri/tauri.conf.json` içindeki sürümü güncelle.
3. Commit et, ardından `vX.Y.Z` etiketini push et. GitHub Actions paketleri derleyip taslak (draft) bir release açar.

## [Unreleased]

### Belgeler

- macOS'taki "Apple could not verify…" uyarısı için kurulum talimatı düzeltildi: macOS 15 ve sonrasında **Sistem Ayarları → Gizlilik ve Güvenlik → Yine de Aç** ya da `xattr` komutu gerekiyor. Aynı not artık her release sayfasında da var.

## [0.1.1] - 2026-11-15

### Değişti

- Buy Me a Coffee bağlantısı `buymeacoffee.com/mustafaerbay` olarak güncellendi. Değişiklik menüdeki `☕ Don'a kahve ısmarla` öğesinde, README'de, GitHub Sponsor düğmesinde ve release notlarında geçerli.

## [0.1.0] - 2026-11-15

İlk sürüm. Don purosunu içtiği sürece bilgisayarın uykusu gelmez.

### Eklendi

- **Uyku engelleme (macOS ve Windows).** Puro yanıkken hem sistem hem ekran uykusu engellenir. Menüdeki `Ekran kararabilir` seçeneği yalnızca ekranın kararmasına izin verir. Uygulama her açılışta puro yanık başlar.
  - macOS'ta IOKit kullanılır; `pmset -g assertions` ile görülebilir.
  - Windows'ta `SetThreadExecutionState` kullanılır; `powercfg /requests` ile görülebilir.
- **Ekranda Don.** Şeffaf, her zaman üstte duran, tüm masaüstlerinde ve tam ekran uygulamaların üstünde görünen, sürüklenebilir bir karakter.
  - Puroyu ağzına götürür, nefes çeker (puronun ucu parlar), dumanı üfler.
  - Çift tıklamak puroyu yakar (kibrit alevi) ya da söndürür (küllüğe bastırır, Don uyuklar: gözler kapanır, "z" uçar).
- **Üç tema:** Tombul Don (varsayılan), Sinsi Don, Sert Don. Hepsi bu proje için özgün olarak çizildi.
- **Menü çubuğu / sistem tepsisi ikonu.** Puro yanıkken 8 saniyede bir küçük bir puf yapar, sönünce gözlerini kapatır. `Hareketli ikon` kapatılırsa ikon tamamen sabit kalır.
- **Menü:** `Puroyu yak/söndür`, `Don'u göster/gizle`, `Ekran kararabilir`, `Don'a dokunulmasın` (tıklamalar Don'un içinden geçer), `Hareketli ikon`, `Boyut` (Küçük / Orta / Büyük), `Tema`, `Açılışta başlat`, `☕ Don'a kahve ısmarla`, `Çıkış`.
- **Ayarlar hatırlanır:** konum, boyut, tema, görünürlük ve seçenekler. Monitörü çıkarılmış ya da ekrandan taşmış bir Don, kullanılabilir alana geri getirilir.
- **Windows:** Tepsi ikonuna sol tık puroyu yakar ya da söndürür, sağ tık menüyü açar. İlk açılışta ikonu görev çubuğuna sabitleme ipucu gösterilir.

### Performans

- Don gizliyken animasyon tamamen durur (işlemci kullanımı 0).
- Don görünürken animasyon saniyede 8 kare çizilir. M serisi bir Mac'te toplam işlemci kullanımı yaklaşık %5'tir.

### Bilinen sınırlamalar

- Paketler imzasız. macOS'ta ilk açılışta sağ tık → **Aç** gerekir; Windows'ta SmartScreen uyarısında **Yine de çalıştır** seçilmelidir.
- Windows sürümü CI'da derlenip test edildi, ancak gerçek bir Windows masaüstünde elle denenmedi.

[Unreleased]: https://github.com/mustafaerbay/keepmeawake/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/mustafaerbay/keepmeawake/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/mustafaerbay/keepmeawake/releases/tag/v0.1.0
