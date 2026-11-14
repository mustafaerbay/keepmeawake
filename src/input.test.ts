// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { attachInput } from "./input";

function ptr(el: Element, type: string, x = 10, y = 10) {
  el.dispatchEvent(new MouseEvent(type, { bubbles: true, clientX: x, clientY: y, button: 0 }));
}

describe("attachInput", () => {
  let el: HTMLElement;
  let onDoubleClick: ReturnType<typeof vi.fn<() => void>>;
  let startDrag: ReturnType<typeof vi.fn<() => void>>;
  beforeEach(() => {
    vi.useFakeTimers();
    el = document.createElement("div");
    document.body.append(el);
    onDoubleClick = vi.fn<() => void>();
    startDrag = vi.fn<() => void>();
    attachInput(el, { onDoubleClick, startDrag, dragThresholdPx: 4 });
  });
  afterEach(() => vi.useRealTimers());

  it("double_click_calls_toggle_not_drag", () => {
    ptr(el, "pointerdown"); ptr(el, "pointerup");
    vi.advanceTimersByTime(150);
    ptr(el, "pointerdown"); ptr(el, "pointerup");
    expect(onDoubleClick).toHaveBeenCalledTimes(1);
    expect(startDrag).not.toHaveBeenCalled();
  });

  it("slow_clicks_are_not_double_click", () => {
    ptr(el, "pointerdown"); ptr(el, "pointerup");
    vi.advanceTimersByTime(400);
    ptr(el, "pointerdown"); ptr(el, "pointerup");
    expect(onDoubleClick).not.toHaveBeenCalled();
  });

  it("move_beyond_threshold_starts_drag", () => {
    ptr(el, "pointerdown", 10, 10);
    ptr(el, "pointermove", 12, 12);
    expect(startDrag).not.toHaveBeenCalled();
    ptr(el, "pointermove", 15, 10);
    ptr(el, "pointermove", 30, 10);
    expect(startDrag).toHaveBeenCalledTimes(1);
  });

  it("drag_then_click_is_not_double_click", () => {
    ptr(el, "pointerdown", 10, 10); ptr(el, "pointermove", 30, 10); ptr(el, "pointerup", 30, 10);
    vi.advanceTimersByTime(100);
    ptr(el, "pointerdown"); ptr(el, "pointerup");
    expect(onDoubleClick).not.toHaveBeenCalled();
  });

  it("move_without_press_does_nothing", () => {
    ptr(el, "pointermove", 50, 50);
    expect(startDrag).not.toHaveBeenCalled();
  });
});
