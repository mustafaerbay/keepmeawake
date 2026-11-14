import sharedCss from "../themes/_shared/smoker.css?raw";

export interface ThemeManifest {
  id: string;
  name: string;
  order?: number;
  window: { width: number; height: number };
  tray: { fps: number; awakeFrames: number; viewBox: string };
  transitions: { lightingMs: number; snuffingMs: number };
}
export type Transition = "lighting" | "snuffing" | null;
export interface Theme { manifest: ThemeManifest; svg: string; css: string }

const manifests = import.meta.glob<ThemeManifest>("../themes/*/theme.json", { eager: true, import: "default" });
const svgs = import.meta.glob<string>("../themes/*/character.svg", { query: "?raw", import: "default" });
const csss = import.meta.glob<string>("../themes/*/character.css", { query: "?raw", import: "default" });

const byId = new Map(Object.entries(manifests).map(([path, m]) => [m.id, { m, dir: path.replace(/theme\.json$/, "") }]));

export function themeIds(): string[] {
  return [...byId.values()].sort((a, b) => (a.m.order ?? 99) - (b.m.order ?? 99)).map((t) => t.m.id);
}

export async function loadTheme(id: string): Promise<Theme> {
  const t = byId.get(id) ?? byId.get(themeIds()[0])!;
  const [svg, css] = await Promise.all([svgs[t.dir + "character.svg"](), csss[t.dir + "character.css"]()]);
  return { manifest: t.m, svg, css: sharedCss + "\n" + css };
}

const timers = new WeakMap<Element, ReturnType<typeof setTimeout>>();

export function applyState(root: Element, awake: boolean, transition: Transition, manifest: ThemeManifest): void {
  root.classList.add("don-root");
  root.classList.toggle("awake", awake);
  root.classList.toggle("asleep", !awake);
  if (!transition) return;
  clearTimeout(timers.get(root));
  root.classList.remove("lighting", "snuffing");
  root.classList.add(transition);
  const ms = transition === "lighting" ? manifest.transitions.lightingMs : manifest.transitions.snuffingMs;
  timers.set(root, setTimeout(() => root.classList.remove(transition), ms));
}

export function setPaused(root: Element, paused: boolean): void {
  root.classList.toggle("paused", paused);
}
