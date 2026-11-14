import { describe, it, expect } from "vitest";
import { clampPosition } from "./position";

const primary = { x: 0, y: 0, w: 1440, h: 900 };
const second = { x: 1440, y: 0, w: 1920, h: 1080 };
const size = { w: 200, h: 260 };

describe("clampPosition", () => {
  it("keeps_onscreen_position", () => {
    expect(clampPosition({ x: 100, y: 100 }, size, [primary, second], primary)).toEqual({ x: 100, y: 100 });
    expect(clampPosition({ x: 2000, y: 500 }, size, [primary, second], primary)).toEqual({ x: 2000, y: 500 });
  });

  it("clamps_offscreen_position", () => {
    // was on a monitor that is now unplugged
    expect(clampPosition({ x: 3600, y: 200 }, size, [primary], primary)).toEqual({
      x: primary.x + primary.w - size.w - 24,
      y: primary.y + primary.h - size.h - 24,
    });
  });

  it("clamps_when_mostly_offscreen", () => {
    // only a sliver left on screen counts as lost
    expect(clampPosition({ x: 1420, y: 100 }, size, [primary], primary)).toEqual({ x: 1216, y: 616 });
  });
});
