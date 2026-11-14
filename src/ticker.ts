interface Steppable { currentTime: number | null | CSSNumberish; pause(): void }
interface AnimRoot { getAnimations(opts?: { subtree: boolean }): Steppable[] }

/** WebKit repaints animated SVG at 60 fps, which costs ~10% CPU for an
 *  idle desk toy. Instead we pause every CSS animation/transition and step
 *  it ourselves at a low, cartoon-ish frame rate. Returns a stop function. */
export function startTicker(root: AnimRoot, fps: number, now: () => number = () => performance.now()): () => void {
  const startedAt = new WeakMap<Steppable, number>();
  const tick = () => {
    const t = now();
    for (const a of root.getAnimations({ subtree: true })) {
      if (!startedAt.has(a)) {
        a.pause();
        startedAt.set(a, t - Number(a.currentTime ?? 0));
      }
      a.currentTime = t - startedAt.get(a)!;
    }
  };
  tick();
  const id = setInterval(tick, 1000 / fps);
  return () => clearInterval(id);
}
