import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { availableMonitors, getCurrentWindow, primaryMonitor, PhysicalPosition } from "@tauri-apps/api/window";
import { applyState, loadTheme, setPaused, type ThemeManifest, type Transition } from "./theme";
import { attachInput } from "./input";
import { clampPosition } from "./position";
import { startTicker } from "./ticker";

const FPS = 8;

interface State { awake: boolean; displayMaySleep: boolean; clickThrough: boolean }

const root = document.getElementById("don")!;
const style = document.head.appendChild(document.createElement("style"));
let manifest: ThemeManifest;
let awake = true;
let stopTicker: (() => void) | null = null;

/** Animate only while Don can be seen; hidden means zero animation work. */
function setVisible(visible: boolean) {
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

/** Bring Don back if he was left on a monitor that is gone. */
async function keepOnScreen() {
  const win = getCurrentWindow();
  const [pos, size, monitors, primary] = await Promise.all([
    win.outerPosition(), win.outerSize(), availableMonitors(), primaryMonitor(),
  ]);
  if (!primary) return;
  const rect = (m: { position: { x: number; y: number }; size: { width: number; height: number } }) =>
    ({ x: m.position.x, y: m.position.y, w: m.size.width, h: m.size.height });
  const next = clampPosition({ x: pos.x, y: pos.y }, { w: size.width, h: size.height }, monitors.map(rect), rect(primary));
  if (next.x !== pos.x || next.y !== pos.y) await win.setPosition(new PhysicalPosition(next.x, next.y));
}

async function main() {
  const state = await invoke<State>("get_state");
  awake = state.awake;
  await mountTheme(await invoke<string>("get_theme"));

  await listen<{ state: State; transition: Transition }>("state-changed", (e) => {
    awake = e.payload.state.awake;
    applyState(root, awake, e.payload.transition, manifest);
  });
  await listen<{ visible: boolean }>("don-visibility", (e) => setVisible(e.payload.visible));
  await listen<{ id: string }>("theme-changed", (e) => mountTheme(e.payload.id));
  document.addEventListener("visibilitychange", () => setVisible(!document.hidden));
  setVisible(!document.hidden);

  attachInput(root, {
    onDoubleClick: () => void invoke("toggle_awake"),
    startDrag: () => void getCurrentWindow().startDragging(),
  });
  await keepOnScreen();
}

main();
