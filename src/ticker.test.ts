import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { startTicker } from "./ticker";

class FakeAnim {
  currentTime: number | null = 0;
  paused = false;
  pause() { this.paused = true; }
}

describe("startTicker", () => {
  let t = 0;
  const now = () => t;
  beforeEach(() => { vi.useFakeTimers(); t = 0; });
  afterEach(() => vi.useRealTimers());

  it("pauses_animations_and_steps_them_at_fps", () => {
    const a = new FakeAnim();
    const root = { getAnimations: () => [a] };
    const stop = startTicker(root, 10, now);
    expect(a.paused).toBe(true);
    t = 100; vi.advanceTimersByTime(100);
    expect(a.currentTime).toBe(100);
    t = 250; vi.advanceTimersByTime(100);
    expect(a.currentTime).toBe(250);
    stop();
  });

  it("new_animations_start_from_their_own_current_time", () => {
    const anims: FakeAnim[] = [];
    const root = { getAnimations: () => anims };
    const stop = startTicker(root, 10, now);
    t = 1000; vi.advanceTimersByTime(100);
    const b = new FakeAnim(); b.currentTime = 0; anims.push(b);
    t = 1100; vi.advanceTimersByTime(100); // first seen at 1100
    expect(b.paused).toBe(true);
    expect(b.currentTime).toBe(0);
    t = 1300; vi.advanceTimersByTime(100);
    expect(b.currentTime).toBe(200);
    stop();
  });

  it("stop_halts_stepping", () => {
    const a = new FakeAnim();
    const stop = startTicker({ getAnimations: () => [a] }, 10, now);
    stop();
    t = 500; vi.advanceTimersByTime(500);
    expect(a.currentTime).toBe(0);
  });
});
