export interface Rect { x: number; y: number; w: number; h: number }
export interface Point { x: number; y: number }

const MARGIN = 24;

/** If the window's centre is not on any monitor (e.g. that monitor was
 *  unplugged), park Don in the bottom-right corner of the primary monitor. */
export function clampPosition(pos: Point, size: { w: number; h: number }, monitors: Rect[], primary: Rect): Point {
  const cx = pos.x + size.w / 2;
  const cy = pos.y + size.h / 2;
  const visible = monitors.some((m) => cx >= m.x && cx < m.x + m.w && cy >= m.y && cy < m.y + m.h);
  if (visible) return pos;
  return { x: primary.x + primary.w - size.w - MARGIN, y: primary.y + primary.h - size.h - MARGIN };
}
