export interface InputOptions {
  onDoubleClick(): void;
  startDrag(): void;
  dragThresholdPx?: number;
  doubleClickMs?: number;
}

/** Distinguishes "double-click Don" (toggle) from "drag Don" (move window).
 *  We start the native drag ourselves instead of data-tauri-drag-region,
 *  because that region turns double-click into maximize. */
export function attachInput(el: Element, opts: InputOptions): void {
  const threshold = opts.dragThresholdPx ?? 4;
  const dblMs = opts.doubleClickMs ?? 300;
  let down: { x: number; y: number } | null = null;
  let dragged = false;
  let lastClick = -Infinity;

  el.addEventListener("pointerdown", (e) => {
    const ev = e as MouseEvent;
    if (ev.button !== 0) return;
    down = { x: ev.clientX, y: ev.clientY };
    dragged = false;
  });
  el.addEventListener("pointermove", (e) => {
    const ev = e as MouseEvent;
    if (!down || dragged) return;
    if (Math.hypot(ev.clientX - down.x, ev.clientY - down.y) > threshold) {
      dragged = true;
      opts.startDrag();
    }
  });
  el.addEventListener("pointerup", () => {
    if (!down) return;
    down = null;
    if (dragged) {
      lastClick = -Infinity;
      return;
    }
    const now = Date.now();
    if (now - lastClick <= dblMs) {
      lastClick = -Infinity;
      opts.onDoubleClick();
    } else {
      lastClick = now;
    }
  });
}
