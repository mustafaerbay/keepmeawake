# keepmeawake Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri v2 tray app for macOS and Windows. It prevents sleep while an original, animated mafia-boss character ("Don") smokes his cigar. The app has an animated tray icon and an optional floating character window.

**Architecture:** The Rust side owns the single source of truth (`Controller`: awake / display-may-sleep / click-through). It drives the OS power assertion through a `Power` trait with one implementation per platform, and drives the tray icon animation. It pushes state to the web front end through Tauri events. The front end has no logic beyond rendering. It loads the selected theme (SVG + CSS) and toggles state classes on the root element.

**Tech Stack:** Tauri 2, Rust (stable), `windows-sys` (Windows), IOKit FFI (macOS), TypeScript + Vite (no framework), Vitest, `tauri-plugin-store`, `tauri-plugin-autostart`, `tauri-plugin-window-state`, `@resvg/resvg-js` (build-time PNG rendering), pnpm.

**Spec:** `docs/superpowers/specs/2026-11-14-keepmeawake-design.md`

## Global Constraints

- Platforms: macOS (Apple Silicon + Intel), Windows 10/11. No Linux.
- Default on launch: awake = true. Awake state is never persisted.
- Default: both system and display sleep are prevented. "Ekran kararabilir" (default off) only allows the display to sleep.
- Character must be original. Never use: "Godfather", "Corleone", the puppeteer-hand logo, any actor likeness, film quotes or music. Character name: **Don**.
- Idle CPU < 2% with the character window visible. Character animations fully paused while the window is hidden.
- macOS: `ActivationPolicy::Accessory` (no Dock icon), `macOSPrivateApi: true`.
- Tray: macOS gets one colored (non-template) PNG per frame, ~36 px tall. Windows gets PNGs per DPI size 16/20/24/32. 4–6 fps while awake, one static frame while asleep.
- Click mapping: macOS tray click opens the menu. Windows tray left-click toggles, right-click opens the menu.
- Character window: ~200×260 px, transparent, no frame or shadow, always on top, skips taskbar, visible on all Spaces and over fullscreen apps (macOS). Drag uses `startDragging()`, not `data-tauri-drag-region`.
- Windows power calls run only on one dedicated, long-lived thread.
- Menu copy (exact): `🔥 Puroyu yak` / `💨 Puroyu söndür`, `Don'u göster` / `Don'u gizle`, `Ekran kararabilir`, `Don'a dokunulmasın`, `Tema ▸` → `Don`, `Açılışta başlat`, `Çıkış`.
- Windows first-run hint (exact): `Don'u görev çubuğuna sabitlemek için ikonu sürükle`.
- Out of scope: timer, signing/notarization, auto-update, WidgetKit, external themes, per-pixel click-through.

## Review Focus

1. **Rapid toggling** (double-click spam on Don or tray): no leaked or doubled assertion, and the final state matches the last action. Pinned by `controller::tests::rapid_toggle_balances_power_calls` (Task 2).
2. **Changing "Ekran kararabilir" while awake:** the assertion is swapped to the new flags with no window where nothing is held (acquire the new one before releasing the old). Pinned by `controller::tests::display_flag_change_reacquires_before_release` (Task 2).
3. **Quitting while awake (menu Çıkış or app drop):** the assertion is released. Pinned by `controller::tests::drop_releases_power` (Task 2).
4. **Saved window position is off-screen** (monitor unplugged): on startup Don is moved to the bottom-right of the primary monitor. Pinned by `position.test.ts::clamps_offscreen_position` (Task 6).
5. **System sleeps by lid close, then wakes:** the app must still show and hold "awake". Manual check in Task 8 (`pmset -g assertions` after wake).

---

## File Structure

```
package.json, vite.config.ts, tsconfig.json, index.html
scripts/render-tray.mjs           # themes/*/tray/*.svg → PNG frames per platform size
themes/don/theme.json             # theme manifest
themes/don/character.svg          # layered character
themes/don/character.css          # state-class animations
themes/don/tray/awake-{0..N}.svg, asleep.svg   # tray source art
themes/don/tray/out/...           # generated PNGs (committed)
src/main.ts                       # boot: load theme, subscribe to events, wire input
src/theme.ts                      # theme manifest types + loader + applyState
src/position.ts                   # off-screen clamp logic (pure)
src/input.ts                      # drag vs double-click handling
src/*.test.ts                     # vitest
src-tauri/src/main.rs             # calls lib::run()
src-tauri/src/lib.rs              # builder, plugins, setup, commands
src-tauri/src/controller.rs       # Controller<P: Power> — pure state machine + tests
src-tauri/src/power/mod.rs        # Power trait + platform selection
src-tauri/src/power/macos.rs      # IOPMAssertion FFI
src-tauri/src/power/windows.rs    # SetThreadExecutionState on dedicated thread
src-tauri/src/tray.rs             # tray icon, menu, animation thread
src-tauri/src/window.rs           # character window setup
src-tauri/src/settings.rs         # store keys + load/save
src-tauri/src/theme_assets.rs     # include_bytes! tray frames per theme
```

---

### Task 1: Toolchain and scaffold

**Files:**
- Create: Tauri v2 vanilla-ts scaffold at repo root (`src/`, `src-tauri/`, `package.json`, …)
- Modify: `src-tauri/tauri.conf.json`, `.gitignore`

**Interfaces:**
- Produces: `pnpm tauri dev` launches a Dockless app. `lib::run()` is the entry point.

- [ ] **Step 1:** Install Rust via rustup (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y`). Verify with `cargo --version`. Ask the user before running, because it installs to their home directory.
- [ ] **Step 2:** Scaffold in place: `pnpm create tauri-app@latest . --template vanilla-ts --manager pnpm --identifier com.keepmeawake.app` (keep `README.md`, `docs/`). Add `.DS_Store`, `node_modules`, `src-tauri/target` to `.gitignore`.
- [ ] **Step 3:** In `tauri.conf.json`: set `app.macOSPrivateApi: true`. Replace the default window with label `don`: `transparent: true, decorations: false, shadow: false, alwaysOnTop: true, skipTaskbar: true, resizable: false, width: 200, height: 260, visible: false`. In `setup`, call `app.set_activation_policy(tauri::ActivationPolicy::Accessory)` under `#[cfg(target_os = "macos")]`.
- [ ] **Step 4:** Run `pnpm tauri dev`. Expected: it compiles, no Dock icon appears, and there is no visible window.
- [ ] **Step 5:** Add Vitest (`pnpm add -D vitest`, script `"test": "vitest run"`). Run `pnpm test`. Expected: "No test files found" exits cleanly (or add `--passWithNoTests`).
- [ ] **Step 6:** Commit `chore: scaffold Tauri v2 app`.

### Task 2: Controller state machine (pure Rust, TDD)

**Files:**
- Create: `src-tauri/src/controller.rs`, `src-tauri/src/power/mod.rs`
- Test: inline `#[cfg(test)] mod tests` in `controller.rs`

**Interfaces:**
- Produces:
  ```rust
  // power/mod.rs
  pub trait Power: Send { fn acquire(&mut self, display: bool) -> Result<(), String>; fn release(&mut self); }
  // controller.rs
  #[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct State { pub awake: bool, pub display_may_sleep: bool, pub click_through: bool }
  pub struct Controller<P: Power> { /* power, state */ }
  impl<P: Power> Controller<P> {
      pub fn new(power: P, display_may_sleep: bool, click_through: bool) -> Self; // starts awake, acquires
      pub fn state(&self) -> State;
      pub fn set_awake(&mut self, awake: bool) -> State;
      pub fn toggle(&mut self) -> State;
      pub fn set_display_may_sleep(&mut self, v: bool) -> State;
      pub fn set_click_through(&mut self, v: bool) -> State;
  }
  impl<P: Power> Drop for Controller<P> // releases if awake
  ```
  `acquire(display)`: `display == true` means the display is also kept on (i.e. `!display_may_sleep`). Each `Power` impl holds at most one assertion. Calling `acquire` while one is held replaces it: the impl creates the new one first, then releases the old one.

- [ ] **Step 1: Write failing tests** using a `FakePower { log: Arc<Mutex<Vec<String>>> }` that records `"acquire(true)"` / `"release"`:
  - `new_starts_awake_and_acquires`: log == `["acquire(true)"]`, `state().awake == true`.
  - `toggle_off_releases`: after `toggle()`, log ends with `"release"`, `awake == false`.
  - `set_awake_same_value_is_noop`: `set_awake(true)` on an awake controller adds nothing to the log.
  - `rapid_toggle_balances_power_calls`: 101 toggles from new → `awake == false`; number of acquires == number of releases.
  - `display_flag_change_reacquires_before_release`: while awake, `set_display_may_sleep(true)` appends exactly `["acquire(false)"]` (the impl swaps internally, so the controller does no release). While asleep it appends nothing, and the next `set_awake(true)` logs `"acquire(false)"`.
  - `drop_releases_power`: drop an awake controller → log ends with `"release"`. Drop an asleep one → no extra release.
  - `acquire_failure_reports_asleep`: if `FakePower` is set to fail, `set_awake(true)` returns `awake == false` (the icon must not lie).
- [ ] **Step 2:** Run `cargo test --manifest-path src-tauri/Cargo.toml controller`. Expected: FAIL (does not compile).
- [ ] **Step 3:** Implement `Controller` and the `Power` trait as specified.
- [ ] **Step 4:** Run the same command. Expected: 7 passed.
- [ ] **Step 5:** Commit `feat: add awake controller state machine`.

### Task 3: Platform power backends

**Files:**
- Create: `src-tauri/src/power/macos.rs`, `src-tauri/src/power/windows.rs`
- Modify: `src-tauri/src/power/mod.rs` (add `pub fn platform() -> impl Power`), `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: `Power` trait (Task 2).
- Produces: `power::platform() -> PlatformPower` (type alias per OS).

- [ ] **Step 1: macOS.** FFI to IOKit (`#[link(name = "IOKit", kind = "framework")]`) plus the `core-foundation` crate for the CFString. `acquire(display)`: create `"PreventUserIdleSystemSleep"`. If `display` is true, also create `"PreventUserIdleDisplaySleep"`. Use the name `"keepmeawake: Don is smoking"`. Create the new assertion IDs before releasing the old ones. `release`: `IOPMAssertionRelease` on each held ID.
- [ ] **Step 2: Windows.** Use the `windows-sys` crate (feature `Win32_System_Power`). `WindowsPower` owns a `std::thread` that loops on an `mpsc::Receiver<Cmd>` (`Acquire(bool)`, `Release`, `Shutdown`) and calls `SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | (display ? ES_DISPLAY_REQUIRED : 0))` or `SetThreadExecutionState(ES_CONTINUOUS)`. `Drop` sends `Shutdown` and joins. A return value of 0 is reported back as an `Err` (use a reply channel).
- [ ] **Step 3:** Add a test binary `src-tauri/examples/hold.rs` that acquires for 15 s, then releases. Run `cargo run --example hold` and, in parallel, `pmset -g assertions | grep keepmeawake`. Expected: two lines (system + display) while it holds, none after release.
- [ ] **Step 4:** Check that it compiles for Windows: `rustup target add x86_64-pc-windows-msvc && cargo check --target x86_64-pc-windows-msvc --manifest-path src-tauri/Cargo.toml` (if the cross-check fails on missing SDK, record it and defer to Task 8's Windows run).
- [ ] **Step 5:** Commit `feat: add macOS and Windows power backends`.

### Task 4: Don character drafts (user checkpoint)

**Files:**
- Create: `design/drafts/don-a.svg`, `don-b.svg`, `don-c.svg`, `design/drafts/index.html` (side-by-side preview, each draft also shown at 36 px height for the tray)

**Interfaces:**
- Produces: the chosen draft, which becomes the base for `themes/don/character.svg` in Task 5.

- [ ] **Step 1:** Draw three original vector drafts in the spec style (bold outlines, cartoonish, dark suit, fedora, white shirt, burgundy accent, cigar with an orange tip). Vary them meaningfully, e.g. A = round and cuddly, B = tall and sly, C = chunky and stern. Respect Global Constraints on originality.
- [ ] **Step 2:** Open `design/drafts/index.html` in the browser pane and screenshot it. Check each one reads at tray size.
- [ ] **Step 3:** **STOP. Show the user and get their pick (and any tweaks).** Do not continue until they choose.
- [ ] **Step 4:** Commit `design: add Don character drafts` (record the chosen draft in the commit message).

### Task 5: Theme contract and Don animations

**Files:**
- Create: `themes/don/theme.json`, `themes/don/character.svg`, `themes/don/character.css`, `src/theme.ts`
- Test: `src/theme.test.ts`

**Interfaces:**
- Produces:
  ```ts
  export interface ThemeManifest { id: string; name: string; window: { width: number; height: number };
    tray: { fps: number; awakeFrames: number }; transitions: { lightingMs: number; snuffingMs: number } }
  export type Transition = "lighting" | "snuffing" | null;
  export function loadTheme(id: string): Promise<{ manifest: ThemeManifest; svg: string; css: string }>;
  export function applyState(root: Element, awake: boolean, transition: Transition, manifest: ThemeManifest): void;
  export function setPaused(root: Element, paused: boolean): void;
  ```
  Themes are imported with Vite `import.meta.glob('/themes/*/{theme.json,character.svg,character.css}', { query: '?raw' })`.
- SVG contract: groups with ids `body`, `head`, `arm`, `cigar`, `tip`, `smoke-1`…`smoke-5`. The root `<svg>` gets class `awake` | `asleep`, plus a temporary `lighting` | `snuffing`. `setPaused` toggles class `paused`, and the CSS sets `.paused * { animation-play-state: paused }`.

- [ ] **Step 1: Failing tests** (jsdom env):
  - `applyState_sets_awake_class`: `awake=true` → root has `awake`, not `asleep`.
  - `applyState_transition_is_temporary`: with `"lighting"` and fake timers, the class is present, then gone after `manifest.transitions.lightingMs`.
  - `applyState_new_transition_cancels_old`: `lighting` then immediately `snuffing` → only `snuffing` is present. When its timer ends, neither is present.
  - `setPaused_toggles_class`.
  - `loadTheme_don_has_required_ids`: the parsed SVG contains every id in the contract.
- [ ] **Step 2:** Run `pnpm test`. Expected: FAIL.
- [ ] **Step 3:** Implement `src/theme.ts`. Create `theme.json` (`id: "don", name: "Don", window 200×260, tray fps 5, awakeFrames 6, lightingMs 1200, snuffingMs 1000`). Build `character.svg` from the chosen draft with the contract ids.
- [ ] **Step 4:** Write `character.css`. Awake loop, 8 s: raise the arm/cigar (0–15%), puff with the tip glowing (15–30%), lower (30–40%), exhale with smoke parts rising/growing/fading with a small sideways sway and staggered `animation-delay`s (40–85%), rest. Asleep: arm lowered, tip grey, no smoke, slight slump and slow "bored" blink. Lighting: a short match flare at the tip. Snuffing: cigar pressed down, one last wisp. Animate only `transform` and `opacity`.
- [ ] **Step 5:** Run `pnpm test`. Expected: 5 passed.
- [ ] **Step 6:** Preview with `pnpm vite` in the browser pane, plus a temporary dev toggle button (`?debug=1` only). Check the awake loop, asleep pose and both transitions. Show the user a screenshot or GIF.
- [ ] **Step 7:** Commit `feat: add theme loader and Don animations`.

### Task 6: Character window

**Files:**
- Create: `src/main.ts` (replace scaffold), `src/input.ts`, `src/position.ts`, `src-tauri/src/window.rs`, `src-tauri/src/lib.rs` (commands + events)
- Test: `src/position.test.ts`, `src/input.test.ts`

**Interfaces:**
- Consumes: `Controller`/`State` (Task 2), `power::platform()` (Task 3), `theme.ts` (Task 5).
- Produces:
  - Managed state: `AppCtl = Mutex<Controller<PlatformPower>>`.
  - Commands: `get_state() -> State`, `toggle_awake() -> State`.
  - Events (Rust → JS): `"state-changed"` with payload `{ state: State, transition: "lighting" | "snuffing" | null }`, and `"don-visibility"` with `{ visible: bool }`.
  - `pub fn emit_state(app: &AppHandle, s: State, transition: Option<&str>)` in `lib.rs`, used by Task 7.
  - `pub fn set_don_visible(app: &AppHandle, visible: bool)` in `window.rs`.
  - `clampPosition(pos: {x,y}, size: {w,h}, monitors: {x,y,w,h}[], primary: {x,y,w,h}): {x,y}` in `position.ts`.
  - `attachInput(el: Element, opts: { onDoubleClick(): void; startDrag(): void; dragThresholdPx?: number }): void` in `input.ts`.

- [ ] **Step 1: Failing tests:**
  - `position.test.ts::keeps_onscreen_position`: a position inside a monitor is returned unchanged.
  - `position.test.ts::clamps_offscreen_position`: a position outside every monitor → `{ x: primary.x + primary.w - size.w - 24, y: primary.y + primary.h - size.h - 24 }`.
  - `input.test.ts::double_click_calls_toggle_not_drag`: two pointerdown/up pairs within 300 ms and no movement → `onDoubleClick` once, `startDrag` 0 times.
  - `input.test.ts::move_beyond_threshold_starts_drag`: pointerdown then move 5 px (threshold 4) → `startDrag` once.
- [ ] **Step 2:** Run `pnpm test`. Expected: the new tests FAIL.
- [ ] **Step 3:** Implement `position.ts` and `input.ts`.
- [ ] **Step 4:** `window.rs`: on setup, size the `don` window from the manifest, call `set_visible_on_all_workspaces(true)`, and on macOS set the NSWindow collection behavior to `canJoinAllSpaces | fullScreenAuxiliary`. Register `tauri-plugin-window-state` for position. After restore, ask the front end to run the clamp (or do the equivalent in Rust with `available_monitors()`; either is fine, but the test must cover the logic used). `set_don_visible` shows/hides and emits `don-visibility`.
- [ ] **Step 5:** `main.ts`: load theme `don`, inject the SVG and CSS, `applyState` from `get_state()`, listen to `state-changed` and `don-visibility` (→ `setPaused(!visible)`), plus `document.visibilitychange`. `attachInput` with `onDoubleClick: () => invoke('toggle_awake')` and `startDrag: () => getCurrentWindow().startDragging()`.
- [ ] **Step 6:** `pnpm tauri dev`, then show the window via a temporary call in setup. Verify: transparent background, drag works, double-click toggles with the transition animation, the window survives a Space switch and sits over a fullscreen app.
- [ ] **Step 7:** CPU check: Activity Monitor with Don visible and idle for 60 s. Expected < 2%. Hide the window, then check the WebView process drops to ~0%.
- [ ] **Step 8:** Commit `feat: add floating Don window`.

### Task 7: Tray icon, menu and settings

**Files:**
- Create: `scripts/render-tray.mjs`, `themes/don/tray/*.svg`, `themes/don/tray/out/**` (generated), `src-tauri/src/theme_assets.rs`, `src-tauri/src/tray.rs`, `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `package.json` (script `"tray:render"`)

**Interfaces:**
- Consumes: `AppCtl`, `emit_state`, `set_don_visible` (Task 6).
- Produces:
  - `settings.rs`: `struct Settings { display_may_sleep: bool, click_through: bool, don_visible: bool, theme: String, hint_shown: bool }`, `fn load(app) -> Settings` (defaults: false, false, true, "don", false), `fn save(app, &Settings)`, store file `settings.json`.
  - `theme_assets.rs`: `fn tray_frames(theme: &str) -> TrayFrames { awake: Vec<&'static [u8]>, asleep: &'static [u8] }`, picking `mac/` on macOS and, on Windows, the size nearest `16 * scale_factor`.
  - `tray.rs`: `fn build(app: &AppHandle) -> tauri::Result<()>`, `fn refresh(app: &AppHandle)` (menu labels/checks from current state).

- [ ] **Step 1:** Draw the tray SVGs from the chosen Don (head + cigar crop): 6 awake frames (tip glow pulse + a tiny smoke wisp drifting up) and 1 asleep. `render-tray.mjs` renders them with `@resvg/resvg-js` to `out/mac/*.png` (36 px tall) and `out/win/{16,20,24,32}/*.png`. Run `pnpm tray:render` and commit the output.
- [ ] **Step 2:** Implement `settings.rs` with `tauri-plugin-store` and `tauri-plugin-autostart` (the autostart state is read from the plugin, not the store).
- [ ] **Step 3:** Implement `tray.rs`. Menu items and order exactly as in Global Constraints. The first item's label follows `awake`. The `Don'u göster/gizle` label follows visibility. `Ekran kararabilir`, `Don'a dokunulmasın` and `Açılışta başlat` are `CheckMenuItem`s. `Tema ▸` is a submenu with one checked item `Don`. Icons use `icon_as_template(false)`. Click mapping: macOS `show_menu_on_left_click(true)`; Windows `show_menu_on_left_click(false)`, and `TrayIconEvent::Click { button: Left, button_state: Up }` → toggle. Every state change goes through `AppCtl`, then `emit_state`, then `refresh`, then `settings::save`. `Don'a dokunulmasın` also calls `set_ignore_cursor_events(v)` on the `don` window. `Çıkış` → `app.exit(0)` (the Controller drops and releases).
- [ ] **Step 4:** Animation: a `std::thread` ticking at `1000 / manifest.tray.fps` ms. While awake it cycles `set_icon` through the awake frames. On transition to asleep it sets the asleep frame once and parks on a `Condvar` until awake again (no polling while asleep).
- [ ] **Step 5:** Startup: build the `Controller` with the loaded settings, apply `click_through`, show Don if `don_visible`. On Windows, if `!hint_shown`, show a native notification with the exact hint text, then set `hint_shown = true`.
- [ ] **Step 6:** `pnpm tauri dev`. Verify every menu item, that labels update, that the tray animates while awake and freezes while asleep, and that settings survive a restart while awake still resets to true. Check `pmset -g assertions`: an entry is present when lit, only system (no display) with `Ekran kararabilir`, and none when put out.
- [ ] **Step 7:** Commit `feat: add animated tray, menu and settings`.

### Task 8: Release builds and platform verification

**Files:**
- Modify: `README.md` (what it is, how to build, how to verify, the copyright note on originality)

- [ ] **Step 1:** `pnpm tauri build` on macOS. Expected: a `.app`/`.dmg` in `src-tauri/target/release/bundle/`. Launch the built app (not dev).
- [ ] **Step 2:** macOS checks. The `pmset -g assertions` toggle cycle works. Close the lid for ~1 min, reopen, and check that the assertion and the "lit" state are still there (Review Focus 5). Idle CPU stays < 2% over 5 min.
- [ ] **Step 3:** Windows: build on a Windows machine (or CI `windows-latest`). Run the `powercfg /requests` (admin) cycle: SYSTEM + DISPLAY when lit, SYSTEM only with `Ekran kararabilir`, nothing when put out. Check that left-click toggles, right-click opens the menu, and the first-run hint appears once. If no Windows machine is available, say so explicitly in the handoff. Do not claim it is verified.
- [ ] **Step 4:** Write the README. Commit `docs: add README with build and verify steps`.
