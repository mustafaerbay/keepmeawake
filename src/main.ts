import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { applyState, loadTheme, setPaused, type ThemeManifest } from "./theme";
import { attachInput } from "./input";
import { boot, type DonState } from "./boot";
import { startTicker } from "./ticker";

const FPS = 8;

const root = document.getElementById("don")!;
const style = document.head.appendChild(document.createElement("style"));
let manifest: ThemeManifest | null = null;
let awake = true;
let windowVisible = true;
let stopTicker: (() => void) | null = null;

/** Animate only while Don can be seen; hidden means zero animation work. */
function updateTicker() {
  const visible = windowVisible && !document.hidden;
  setPaused(root, !visible);
  if (visible && !stopTicker) stopTicker = startTicker(root, FPS);
  if (!visible && stopTicker) { stopTicker(); stopTicker = null; }
}

async function mountTheme(id: string) {
  const t = await loadTheme(id);
  manifest = t.manifest;
  style.textContent = t.css;
  root.innerHTML = t.svg;
  applyState(root, awake, null, manifest);
}

boot({
  listen: (event, cb) => listen(event, (e) => cb(e.payload as never)),
  getState: () => invoke<DonState>("get_state"),
  getTheme: () => invoke<string>("get_theme"),
  isVisible: () => getCurrentWindow().isVisible(),
  onState: (s, transition) => {
    awake = s.awake;
    if (manifest) applyState(root, awake, transition, manifest);
  },
  onVisible: (v) => { windowVisible = v; updateTicker(); },
  onTheme: mountTheme,
});

document.addEventListener("visibilitychange", updateTicker);
attachInput(root, {
  onDoubleClick: () => void invoke("toggle_awake"),
  startDrag: () => void getCurrentWindow().startDragging(),
});
