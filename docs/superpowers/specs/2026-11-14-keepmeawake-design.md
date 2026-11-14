# keepmeawake — Tasarım Belgesi

Tarih: 2026-11-14

## Amaç ve neden

macOS ve Windows bilgisayarın uykuya düşmesini engelleyen küçük bir araç. Fark yaratan şey sıkıcı bir aç/kapa düğmesi değil, **eğlenceli olması**: bilgisayarı uyanık tutan şey ekranda yaşayan, animasyonlu bir karakter. İlk tema bir mafya babası (fötr şapka, takım elbise, puro). Ardından başka temalar eklenecek.

Ana metafor: **Karakter purosunu içtiği sürece bilgisayar uyumaz. Puro sönünce uyku normale döner.** *(benim önerim, kullanıcı itiraz etmedi)*

## Hedefler

- macOS ve Windows'ta tek kod tabanından çalışan bir uygulama.
- macOS'ta **menü çubuğunda**, Windows'ta **görev çubuğunun sistem tepsisinde** her zaman görünen minik, animasyonlu bir ikon. Puro ucu yanıyor, minik duman çıkıyor.
- İsteğe bağlı, ekranda serbest duran, **büyük karakter penceresi**: şeffaf arka planlı, çerçevesiz, her zaman üstte, sürüklenebilir.
- Karakter animasyon döngüsü: puroyu ağza götürme → nefes çekme (puro ucu parlıyor) → indirme → duman üfleme (halkalar/bulut yükselip dağılıyor) → kısa bekleme → tekrar. Döngü süresi ~6–10 sn.
- "Söndürülmüş" durum: puro sönük, duman yok. Karakter sıkılmış/uykulu bir pozda duruyor. Tepsi ikonu da sönük versiyona geçiyor.
- **Tema sistemi** baştan kurulacak. Yeni tema eklemek kod değil, içerik işi olmalı (bkz. Tema yapısı).

## Hedef dışı (v1'de yok)

- macOS'un yerleşik WidgetKit widget'ları (masaüstü/bildirim merkezi). Tauri'nin yanına ayrı bir Swift eklentisi gerektirir. C seçeneğinde gerek yok.
- Zamanlayıcı ("2 saat uyanık tut"). İleride eklenebilir. *(benim kararım)*
- Kod imzalama, notarization, otomatik güncelleme, mağaza dağıtımı. v1 yerel derleme/imzasız paket. *(benim kararım)*
- Linux desteği.
- İkinci bir tema. v1 tek temayla çıkar, sistem ikinciye hazır olur.

## Anti-hedefler (çalışsa bile başarısızlık sayılır)

- **Telif:** Karakter Godfather filmini veya herhangi bir gerçek kişiyi/aktörü taklit etmeyecek. Film adı, logo (kuklacı eli), Corleone ismi, aktör yüz benzerliği, film repliği/müziği yok. Karakter özgün: genel "1940'lar mafya babası" estetiği. Çalışma adı **"Don"**.
- **Ağırlık:** Arka planda sürekli duran bir araç pil/CPU yememeli. Hedef: karakter penceresi açıkken bile boşta CPU < %2, pencere gizliyken animasyon tamamen durur.
- **Yalan söyleyen durum:** İkon "yanıyor" gösterirken uyku engellenmiyorsa ya da kullanıcı söndürdükten/uygulamadan çıktıktan sonra engel sürüyorsa bu bir hata. Çıkışta ve çökmede engel serbest bırakılmalı (işletim sistemi süreç bitince zaten bırakır, buna güvenilebilir).

## Kısıtlar

- Platformlar: macOS (Apple Silicon + Intel), Windows 10/11.
- Teknoloji: **Tauri v2** (Rust arka uç + web ön yüz). *(kullanıcı seçti)*

## Yaklaşımlar

| Yaklaşım | Artı | Eksi |
|---|---|---|
| **1. Tauri** *(seçilen)* | Tek kod, iki platform. Temalar SVG/CSS olduğu için ekleme kolay. ~10 MB. | macOS WidgetKit yok. |
| 2. Yerel (Swift + C#) | En yerli his, ileride gerçek widget mümkün. | Her şey ve her tema iki kere yazılır. |
| 3. Electron | Kolay. | ~150 MB, sürekli RAM. Arka plan aracı için ağır. |

Kullanıcı Tauri'yi seçti çünkü tema çoğaltma planına en iyi o uyuyor ve iki platforma tek seferde çıkıyor.

Arayüz yerleşimi için de üç seçenek konuşuldu: sadece tepsi, sadece serbest karakter, ikisi birden. Kullanıcı **ikisi birden (C)** seçeneğini seçti.

## Kullanıcının verdiği kararlar

- İlk tema: mafya babası, puro içme ve duman animasyonu.
- Yerleşim: C (minik tepsi ikonu her zaman + isteğe bağlı büyük karakter).
- Teknoloji: Tauri.
- Çizim tarzı: vektör (SVG).

## Davranış

### Uyku engelleme (Rust)
- **macOS:** `IOPMAssertionCreateWithName` ile `kIOPMAssertionTypePreventUserIdleSystemSleep` ve `kIOPMAssertionTypePreventUserIdleDisplaySleep`. Söndürünce `IOPMAssertionRelease`.
- **Windows:** `SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED)`. Söndürünce `SetThreadExecutionState(ES_CONTINUOUS)`. Bu çağrı iş parçacığına bağlı olduğu için ömür boyu yaşayan tek bir iş parçacığından yapılmalı.
- Varsayılan: hem sistem hem **ekran** uykusu engellenir. Ekranın kararmasını da engellemek, "bilgisayar uyumasın" beklentisine en yakın olanı. *(benim kararım)* Ekranın kararmasına izin veren bir ayar v1'de menüde seçenek olarak bulunur.
- Hazır bir crate kullanılabilir (ör. `keepawake`). Bu seçim kurucuya kalmış, ama yukarıdaki davranışı sağlamalı. Windows'ta engel nesnesi Tauri komut/tokio iş parçacığında değil, bu iş için ayrılmış tek bir iş parçacığında yaşamalı.
- Kapak kapatma uykusu ve kullanıcının elle uyutması engellenmez (işletim sistemi kuralı, hedef de değil).

### Durum
- Tek kaynak: Rust tarafındaki `awake: bool`. Ön yüz bu durumu olaylarla (event) dinler. Tepsi ikonu, karakter animasyonu ve uyku engeli her zaman bu değerden türetilir.
- Uygulama açılışta **yanık** (uyanık) başlar. Uygulamayı açmanın sebebi bu. *(benim kararım)*

### Tepsi / menü çubuğu
- İkon: temanın sağladığı kare dizisi (PNG). Yanıkken ~4–6 fps ile döngü (puro ucu parlıyor, minik duman). Sönükken tek, statik kare.
- macOS'ta ikon renkli (template değil), çünkü puro ucunun turuncu parıltısı karakterin parçası. Açık ve koyu menü çubuğunda okunabilir olmalı. *(benim kararım)*
- Tauri v2 `set_icon` tek görsel alır: macOS için ~36px yüksekliğinde tek yüksek çözünürlüklü kare (18pt'de çizilir), Windows için DPI'ya göre seçilen boyut (16/20/24/32px).
- Tepsi animasyonu yanıkken her zaman döner (karakter penceresi gizli olsa bile). Birkaç küçük PNG'yi saniyede 4–6 kez değiştirmek ucuzdur. *(benim kararım)*
- Tıklama: macOS'ta tık menüyü açar. Windows'ta sol tık yak/söndür arasında geçiş yapar, sağ tık menüyü açar (Windows'ta çift tık menüyle çakıştığı için). *(benim kararım)*
- Windows 11 yeni ikonları taşma alanına (^) gizleyebilir, uygulama kendini sabitleyemez. İlk açılışta bir kerelik ipucu gösterilir: "Don'u görev çubuğuna sabitlemek için ikonu sürükle". *(benim kararım)*
- Menü öğeleri:
  - `🔥 Puroyu yak` / `💨 Puroyu söndür` (durumu değiştirir)
  - `Don'u göster` / `Don'u gizle` (karakter penceresi)
  - `Ekran kararabilir` (onay kutusu, varsayılan kapalı)
  - `Don'a dokunulmasın` (onay kutusu, varsayılan kapalı)
  - `Tema ▸` (v1'de tek öğe, ama alt menü var)
  - `Açılışta başlat` (onay kutusu)
  - `Çıkış`

### Karakter penceresi
- Şeffaf, çerçevesiz, gölgesiz, her zaman üstte, görev çubuğunda/Dock'ta görünmez. macOS'ta şeffaf pencere için `macOSPrivateApi: true` gerekir (App Store'u dışlar; mağaza zaten hedef dışı).
- macOS'ta tüm masaüstlerinde (Spaces) ve tam ekran uygulamaların üstünde görünür (`visible_on_all_workspaces` + koleksiyon davranışı). *(benim kararım: "her zaman orada" hissi için)*
- Tıklama geçirgenliği: pencere kendi dikdörtgenindeki tıklamaları yakalar, şeffaf kısım dahil. Piksel bazında geçirgenlik v1'de yok. Pencere küçük ve kenara taşınabilir. Menüde ek olarak `Don'a dokunulmasın` seçeneği var: açıkken tüm tıklamalar pencereden geçer (`set_ignore_cursor_events`), Don sadece seyirlik olur. *(benim kararım)*
- Boyut ~200×260 px (tema belirler). Sürükleyerek taşınır, konum hatırlanır. Sürükleme `data-tauri-drag-region` ile değil, `startDragging` ile elle başlatılır; aksi halde çift tık pencereyi büyütür ve yak/söndür ile çakışır.
- Karaktere çift tıklamak yak/söndür arasında geçiş yapar. Yanarken çakmak/kibrit kısa animasyonu oynar, söndürürken puro küllüğe bastırılır. *(benim önerim; kurucu kısa tutabilir)*
- İlk açılışta görünür (kullanıcı karakteri hemen görsün). Sonra son durumu hatırlanır.
- Gizliyken animasyon durur (`requestAnimationFrame`/CSS animasyonları duraklatılır).
- macOS: uygulama Dock'ta görünmez (`ActivationPolicy::Accessory`).

### Ayarların saklanması
- `tauri-plugin-store` ile: pencere konumu, pencere görünürlüğü, "ekran kararabilir", seçili tema. Puronun yanık/sönük durumu saklanmaz, her açılışta yanık başlar.
- Açılışta başlatma: `tauri-plugin-autostart`.

## Tema yapısı

```
themes/
  don/
    theme.json        # id, ad, pencere boyutu, tepsi fps, animasyon süreleri
    character.svg     # katmanlı karakter (id'li gruplar: body, arm, cigar, tip, smoke…)
    character.css     # durum sınıflarına bağlı animasyonlar: .awake, .asleep, .lighting, .snuffing
    tray/
      mac/awake-0.png … awake-N.png, asleep.png   # ~36px yükseklik
      win/<16|20|24|32>/awake-0.png …, asleep.png  # DPI başına
```

- Ön yüz, seçili temanın SVG'sini yükleyip kök elemana durum sınıfı (`awake`/`asleep`) ve geçiş sınıfı (`lighting`/`snuffing`) atar. Tüm hareket temanın CSS'inde yaşar. Bu sayede yeni tema eklemek kod değişikliği gerektirmez.
- Duman: SVG içinde birkaç duman parçası. CSS ile yükselme, büyüme, saydamlaşma, hafif yanal salınım; başlangıçlar zaman kaydırmalı. JS parçacık sistemi gerekmez.
- Temalar v1'de uygulamaya gömülü gelir (kullanıcının harici tema yüklemesi hedef dışı).
- Tepsi PNG'leri elle çizilebilir ya da SVG'den bir derleme betiğiyle üretilebilir (kurucuya kalmış).

## Görsel üslup

- Vektör (SVG), sade, karikatürümsü, kalın hatlar. Sıcak tonlar: koyu takım, fötr şapka, beyaz gömlek, kırmızı/bordo detay, puro ucunda turuncu parıltı.
- Tarz: **vektör (SVG)**. *(kullanıcı seçti; pixel-art hedef dışı)*
- Karakteri kurucu özgün olarak çizer. Kesin çizim kullanıcıyla netleşir: kurucu 2–3 vektör karakter taslağını yan yana gösterir, kullanıcı seçer, ardından animasyonlar o taslak üzerine kurulur.

## Doğrulama

- macOS: `pmset -g assertions`. Yanıkken uygulamanın engeli listede görünmeli, söndürünce kaybolmalı.
- Windows: `powercfg /requests` (yönetici komut istemi gerekir). Yanıkken SYSTEM ve DISPLAY altında görünmeli, söndürünce kaybolmalı.
- Rust tarafında durum geçişleri için birim testleri.
- CPU: Activity Monitor / Görev Yöneticisi ile boşta kullanım kontrolü.

## Kurucuya bırakılanlar

- Ön yüz için framework (önerim: framework'süz TypeScript + Vite, çünkü arayüz çok küçük).
- Uyku engelleme için hazır crate mi, doğrudan FFI mı.
- Tepsi karelerinin sayısı, kesin animasyon zamanlamaları, karakter penceresinin kesin boyutu.
- Tepsi PNG'lerinin üretim yöntemi.
- Proje klasör düzeni (Tauri varsayılanı).
