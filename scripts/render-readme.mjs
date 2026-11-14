// Renders README images from the theme SVGs into docs/images/:
//   don-smoking.gif  all themes side by side, one 8 s smoking loop
//   don-asleep.png   all themes put out (dozing)
//   tray.png         tray icons, lit and put out, on light and dark menu bars
// The pose math mirrors themes/_shared/smoker.css keyframes.
import { Resvg } from "@resvg/resvg-js";
import gifenc from "gifenc";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const { GIFEncoder, quantize, applyPalette } = gifenc;
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const OUT = path.join(ROOT, "docs", "images");
fs.mkdirSync(OUT, { recursive: true });

const themes = fs.readdirSync(path.join(ROOT, "themes"))
  .filter((d) => fs.existsSync(path.join(ROOT, "themes", d, "theme.json")))
  .map((d) => {
    const dir = path.join(ROOT, "themes", d);
    const m = JSON.parse(fs.readFileSync(path.join(dir, "theme.json"), "utf8"));
    const css = fs.readFileSync(path.join(dir, "character.css"), "utf8");
    const pt = (name) => css.match(new RegExp(`--${name}:\\s*([\\d.]+)px\\s+([\\d.]+)px`)).slice(1).map(Number);
    const deg = (name) => Number(css.match(new RegExp(`--${name}:\\s*([\\d.]+)deg`))[1]);
    return { ...m, svg: fs.readFileSync(path.join(dir, "character.svg"), "utf8"),
      shoulder: pt("shoulder"), neck: pt("neck"), tip: pt("tip"), rest: deg("rest-angle"), sleep: deg("sleep-angle") };
  })
  .sort((a, b) => (a.order ?? 99) - (b.order ?? 99));

// ---- keyframe helpers (percent of an 8 s loop) ----
const ease = (x) => (x < 0.5 ? 2 * x * x : 1 - (-2 * x + 2) ** 2 / 2);
function track(stops, p, easeFn = ease) {
  if (p <= stops[0][0]) return stops[0][1];
  for (let i = 1; i < stops.length; i++) {
    const [p0, v0] = stops[i - 1], [p1, v1] = stops[i];
    if (p <= p1) {
      const k = p1 === p0 ? 1 : easeFn((p - p0) / (p1 - p0));
      return Array.isArray(v0) ? v0.map((a, j) => a + (v1[j] - a) * k) : v0 + (v1 - v0) * k;
    }
  }
  return stops[stops.length - 1][1];
}
const lin = (x) => x;
const hex = (c) => [1, 3, 5].map((i) => parseInt(c.slice(i, i + 2), 16));
const toHex = (rgb) => "#" + rgb.map((v) => Math.round(v).toString(16).padStart(2, "0")).join("");

function setAttr(svg, id, attrs) {
  const re = new RegExp(`<(g|circle|text)\\b[^>]*\\bid="${id}"[^>]*>`);
  return svg.replace(re, (tag) => {
    let t = tag;
    for (const k of Object.keys(attrs)) t = t.replace(new RegExp(`\\s${k}="[^"]*"`), "");
    const close = t.endsWith("/>") ? "/>" : ">";
    return t.slice(0, -close.length) + Object.entries(attrs).map(([k, v]) => ` ${k}="${v}"`).join("") + close;
  });
}
const circleCenter = (svg, id) => svg.match(new RegExp(`id="${id}"[^>]*cx="([\\d.]+)" cy="([\\d.]+)"`)).slice(1).map(Number);
const scaleAt = ([cx, cy], dx, dy, s) => `translate(${cx + dx} ${cy + dy}) scale(${s}) translate(${-cx} ${-cy})`;

/** The theme SVG posed at time t (seconds) of the awake loop, or asleep. */
function pose(t, theme, asleep = false) {
  let s = theme.svg;
  const [sx, sy] = theme.shoulder, [nx, ny] = theme.neck, [tx, ty] = theme.tip;
  if (asleep) {
    s = setAttr(s, "arm", { transform: `rotate(${theme.sleep} ${sx} ${sy})` });
    s = setAttr(s, "wisps", { opacity: 0 });
    s = setAttr(s, "head", { transform: `rotate(5 ${nx} ${ny})` });
    s = setAttr(s, "lids", { opacity: 1 });
    s = setAttr(s, "zzz", { opacity: 1, transform: "translate(6 -6)" });
    s = setAttr(s, "puffs", { opacity: 0 });
    return s.replace(/(<circle id="tip"[^>]*fill=")[^"]*"/, '$1#8a8a8a"');
  }
  const p = ((t % 8) / 8) * 100;
  const arm = track([[0, theme.rest], [8, theme.rest], [18, 0], [32, 0], [42, theme.rest]], p);
  const head = track([[0, 0], [20, 0], [26, -3], [32, -3], [44, -5], [56, -5], [66, 0]], p);
  const tip = toHex(track([[0, hex("#ff7a1a")], [18, hex("#ff7a1a")], [22, hex("#ffe14a")], [30, hex("#ffe14a")], [36, hex("#ff7a1a")]], p, lin));
  const wispsOn = track([[0, 1], [6, 1], [10, 0], [40, 0], [46, 1]], p, lin);
  s = setAttr(s, "arm", { transform: `rotate(${arm} ${sx} ${sy})` });
  s = setAttr(s, "head", { transform: `rotate(${head} ${nx} ${ny})` });
  s = setAttr(s, "wisps", { opacity: wispsOn, transform: `rotate(${-theme.rest} ${tx} ${ty})` });
  [0, 0.8, 1.6].forEach((delay, i) => {
    const id = `smoke-${i + 1}`;
    const q = ((((t - delay) % 2.4) + 2.4) % 2.4) / 2.4;
    const op = q < 0.15 ? (q / 0.15) * 0.95 : 0.95 * (1 - (q - 0.15) / 0.85);
    const [dx, dy] = track([[0, [0, 0]], [0.5, [-6, -22]], [1, [4, -46]]], q, lin);
    const sc = track([[0, 0.5], [0.5, 1.2], [1, 2]], q, lin);
    s = setAttr(s, id, { opacity: op.toFixed(3), transform: scaleAt(circleCenter(theme.svg, id), dx, dy, sc) });
  });
  [0, 0.35].forEach((delay, i) => {
    const id = `smoke-${i + 4}`;
    const q = ((((t - delay) % 8) + 8) % 8) / 8 * 100;
    const op = track([[0, 0], [40, 0], [43, 0.9], [72, 0]], q, lin);
    const [dx, dy] = track([[0, [0, 0]], [40, [0, 0]], [43, [-4, -2]], [72, [-30, -56]]], q, lin);
    const sc = track([[0, 0.4], [40, 0.4], [43, 0.7], [72, 2.6]], q, lin);
    s = setAttr(s, id, { opacity: op.toFixed(3), transform: scaleAt(circleCenter(theme.svg, id), dx, dy, sc) });
  });
  return s.replace(/(<circle id="tip"[^>]*fill=")[^"]*"/, `$1${tip}"`);
}

// ---- compose a row of Dons on a card ----
const W = 200, H = 260, GAP = 20, PAD = 16, LABEL = 28, BG = "#f3efe6";
const inner = (svg) => svg.replace(/^[\s\S]*?<svg[^>]*>/, "").replace(/<\/svg>\s*$/, "");
function row(svgs, labels) {
  const width = PAD * 2 + svgs.length * W + (svgs.length - 1) * GAP;
  const height = PAD * 2 + H + LABEL;
  const cells = svgs.map((s, i) => {
    const x = PAD + i * (W + GAP);
    return `<svg x="${x}" y="${PAD}" width="${W}" height="${H}" viewBox="0 0 200 260" overflow="visible">${inner(s)}</svg>` +
      `<text x="${x + W / 2}" y="${PAD + H + 20}" text-anchor="middle" font-family="Helvetica, Arial, sans-serif" font-size="15" font-weight="600" fill="#3a3530">${labels[i]}</text>`;
  }).join("");
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><rect width="100%" height="100%" rx="18" fill="${BG}"/>${cells}</svg>`;
}
const render = (svg, zoom) => new Resvg(svg, { fitTo: { mode: "zoom", value: zoom }, font: { loadSystemFonts: true } }).render();

// 1) smoking GIF
const FPS = 10, SECONDS = 8, ZOOM = 1;
const gif = GIFEncoder();
const frames = [];
for (let f = 0; f < FPS * SECONDS; f++) {
  frames.push(render(row(themes.map((th) => pose(f / FPS, th)), themes.map((th) => th.name)), ZOOM));
}
// One palette for the whole loop: per-frame palettes make colours flicker.
const sample = frames.filter((_, i) => i % 8 === 0);
const joined = new Uint8Array(sample.reduce((n, fr) => n + fr.pixels.length, 0));
sample.reduce((off, fr) => (joined.set(fr.pixels, off), off + fr.pixels.length), 0);
const palette = quantize(joined, 256);
frames.forEach((fr, i) => gif.writeFrame(applyPalette(fr.pixels, palette), fr.width, fr.height,
  i === 0 ? { palette, delay: 1000 / FPS } : { delay: 1000 / FPS }));
gif.finish();
fs.writeFileSync(path.join(OUT, "don-smoking.gif"), gif.bytes());

// 2) asleep PNG (2x for crispness)
fs.writeFileSync(path.join(OUT, "don-asleep.png"),
  render(row(themes.map((th) => pose(0, th, true)), themes.map((th) => th.name)), 2).asPng());

// 3) tray strip from the rendered tray PNGs
const b64 = (f) => "data:image/png;base64," + fs.readFileSync(f).toString("base64");
let cells = "", y = 0;
const SZ = 36, BAR_W = 2 * (SZ + 12) + 12;
for (const th of themes) {
  const dir = path.join(ROOT, "themes", th.id, "tray", "out", "mac");
  ["#ececec", "#1e1e1e"].forEach((bg, k) => {
    const x = 16 + k * (BAR_W + 16);
    cells += `<rect x="${x}" y="${16 + y}" width="${BAR_W}" height="${SZ + 12}" rx="8" fill="${bg}"/>`;
    cells += `<image x="${x + 12}" y="${22 + y}" width="${SZ}" height="${SZ}" href="${b64(path.join(dir, "awake-0.png"))}"/>`;
    cells += `<image x="${x + 24 + SZ}" y="${22 + y}" width="${SZ}" height="${SZ}" href="${b64(path.join(dir, "asleep.png"))}"/>`;
  });
  cells += `<text x="${32 + 2 * BAR_W + 16}" y="${46 + y}" font-family="Helvetica, Arial, sans-serif" font-size="15" font-weight="600" fill="#3a3530">${th.name}</text>`;
  y += SZ + 24;
}
const trayW = 32 + 2 * BAR_W + 16 + 110;
fs.writeFileSync(path.join(OUT, "tray.png"), render(
  `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${trayW}" height="${y + 16}"><rect width="100%" height="100%" rx="14" fill="${BG}"/>${cells}</svg>`, 2).asPng());

for (const f of fs.readdirSync(OUT)) console.log(f, (fs.statSync(path.join(OUT, f)).size / 1024).toFixed(0) + " KB");
