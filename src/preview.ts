// Dev-only page to eyeball every theme's animations. Not shipped.
import { applyState, loadTheme, themeIds } from "./theme";

const row = document.getElementById("row")!;
const stages: { el: HTMLElement; awake: boolean; m: Awaited<ReturnType<typeof loadTheme>>["manifest"] }[] = [];
for (const id of themeIds()) {
  const t = await loadTheme(id);
  const card = document.createElement("div");
  card.className = "c";
  card.innerHTML = `<style>${t.css.replaceAll(".don-root", `[data-t="${id}"].don-root`)}</style>
    <div class="stage" data-t="${id}">${t.svg}</div><div>${t.manifest.name}</div><button>yak/söndür</button>`;
  row.append(card);
  const s = { el: card.querySelector<HTMLElement>(".stage")!, awake: true, m: t.manifest };
  applyState(s.el, true, null, s.m);
  const flip = () => { s.awake = !s.awake; applyState(s.el, s.awake, s.awake ? "lighting" : "snuffing", s.m); };
  card.querySelector("button")!.onclick = flip;
  s.el.ondblclick = flip;
  stages.push(s);
}
document.getElementById("all")!.onclick = () => stages.forEach((s) => s.el.dispatchEvent(new MouseEvent("dblclick")));
document.getElementById("bg")!.onclick = () => document.body.classList.toggle("dark");
