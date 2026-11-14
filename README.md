# keepmeawake

Bilgisayarı uyanık tutan, puro içen bir mafya babası. **Don purosunu içtiği sürece Mac'in ya da Windows PC'nin uykusu gelmez.** Puroyu söndürünce uyku düzeni normale döner.

- **Menü çubuğu / sistem tepsisi:** Puro ucu yanan, dumanı tüten minik bir Don ikonu.
- **Ekranda Don:** Şeffaf, hep üstte duran, sürüklenebilir bir karakter. Puroyu ağzına götürür, nefes çeker, dumanı üfler. Söndürünce uyuklar.
- **Üç tema:** Tombul Don (varsayılan), Sinsi Don, Sert Don.

Karakterler bu proje için özgün olarak çizildi. Hiçbir filmden, kişiden veya markadan alıntı içermez.

## Kullanım

| Ne | Nasıl |
|---|---|
| Puroyu yak / söndür | Don'a çift tıkla. Ya da menüden `🔥 Puroyu yak` / `💨 Puroyu söndür`. Windows'ta tepsi ikonuna sol tıkla. |
| Menü | macOS: menü çubuğundaki ikona tıkla. Windows: tepsi ikonuna sağ tıkla. |
| Don'u taşı | Sürükle. Konumu hatırlanır. |
| Don'u gizle | Menü → `Don'u gizle` (gizliyken animasyon tamamen durur). |
| Ekran kararsın ama sistem uyumasın | Menü → `Ekran kararabilir`. |
| Don tıklamaları engellemesin | Menü → `Don'a dokunulmasın` (tıklamalar Don'un içinden geçer). |
| Tema | Menü → `Tema` |
| Açılışta başlat | Menü → `Açılışta başlat` |

Uygulama her açılışta puro **yanık** başlar.

## Geliştirme

Gerekenler: Rust (stable), Node 20+, pnpm.

```bash
pnpm install
pnpm tauri dev          # uygulamayı geliştirme modunda çalıştırır
pnpm test               # ön yüz testleri (vitest)
cargo test --manifest-path src-tauri/Cargo.toml --lib   # Rust testleri
pnpm tauri build        # paket: src-tauri/target/release/bundle/
```

Animasyonları uygulama olmadan izlemek için `pnpm vite` çalıştırıp `http://localhost:1420/preview.html` adresini aç.

### Yeni tema eklemek

1. `themes/<id>/` klasörünü oluştur: `theme.json`, `character.svg`, `character.css`.
2. `character.svg` şu id'leri içermeli: `body`, `head`, `arm` (içinde `cigar`, `tip`, `wisps` → `flame`, `smoke-1..3`), `lids`, `puffs` → `smoke-4..5`, `zzz`. Animasyonun ortak kısmı `themes/_shared/smoker.css` dosyasında. Temanın CSS'i sadece dönme noktalarını (`--shoulder`, `--neck`, `--tip`) ve açıları verir.
3. `pnpm tray:render` komutu tepsi karelerini ve `src-tauri/src/tray_frames.rs` dosyasını yeniden üretir.

## Uykunun gerçekten engellendiğini doğrulama

- **macOS:** `pmset -g assertions | grep keepmeawake` komutunu çalıştır. Puro yanıkken `PreventUserIdleSystemSleep` satırı görünür; `Ekran kararabilir` kapalıysa `PreventUserIdleDisplaySleep` satırı da görünür. Puro sönünce satırlar kaybolur.
- **Windows:** Yönetici komut isteminde `powercfg /requests` çalıştır. Puro yanıkken SYSTEM (ve DISPLAY) altında görünür, sönünce kaybolur.

Kapak kapatma ve elle uyutma engellenmez. Bunlar işletim sisteminin kuralıdır.
