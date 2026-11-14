// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { applyState, setPaused, loadTheme, themeIds, type ThemeManifest } from "./theme";

const manifest: ThemeManifest = {
  id: "t", name: "T", window: { width: 200, height: 260 },
  tray: { fps: 5, awakeFrames: 6, viewBox: "0 0 10 10" },
  transitions: { lightingMs: 1200, snuffingMs: 1000 },
};

const REQUIRED_IDS = ["body", "head", "arm", "cigar", "tip", "lids", "flame",
  "smoke-1", "smoke-2", "smoke-3", "smoke-4", "smoke-5"];

describe("theme", () => {
  let root: Element;
  beforeEach(() => {
    vi.useFakeTimers();
    root = document.createElement("div");
  });
  afterEach(() => vi.useRealTimers());

  it("applyState_sets_awake_class", () => {
    applyState(root, true, null, manifest);
    expect(root.classList.contains("awake")).toBe(true);
    expect(root.classList.contains("asleep")).toBe(false);
    applyState(root, false, null, manifest);
    expect(root.classList.contains("asleep")).toBe(true);
    expect(root.classList.contains("awake")).toBe(false);
  });

  it("applyState_transition_is_temporary", () => {
    applyState(root, true, "lighting", manifest);
    expect(root.classList.contains("lighting")).toBe(true);
    vi.advanceTimersByTime(1199);
    expect(root.classList.contains("lighting")).toBe(true);
    vi.advanceTimersByTime(1);
    expect(root.classList.contains("lighting")).toBe(false);
  });

  it("applyState_new_transition_cancels_old", () => {
    applyState(root, true, "lighting", manifest);
    applyState(root, false, "snuffing", manifest);
    expect(root.classList.contains("lighting")).toBe(false);
    expect(root.classList.contains("snuffing")).toBe(true);
    vi.advanceTimersByTime(1000);
    expect(root.classList.contains("lighting")).toBe(false);
    expect(root.classList.contains("snuffing")).toBe(false);
  });

  it("setPaused_toggles_class", () => {
    setPaused(root, true);
    expect(root.classList.contains("paused")).toBe(true);
    setPaused(root, false);
    expect(root.classList.contains("paused")).toBe(false);
  });

  it("ships_three_don_themes_with_tombul_first", () => {
    expect(themeIds()).toEqual(["don-tombul", "don-sinsi", "don-sert"]);
  });

  it("loadTheme_every_theme_has_required_ids", async () => {
    vi.useRealTimers();
    for (const id of themeIds()) {
      const t = await loadTheme(id);
      expect(t.manifest.id).toBe(id);
      const doc = new DOMParser().parseFromString(t.svg, "image/svg+xml");
      for (const req of REQUIRED_IDS) {
        expect(doc.getElementById(req), `${id} #${req}`).not.toBeNull();
      }
      expect(t.css).toContain("@keyframes");
      // flame and tip smoke must move with the hand holding the cigar
      const arm = doc.getElementById("arm")!;
      for (const req of ["cigar", "tip", "flame", "smoke-1", "smoke-2", "smoke-3"]) {
        expect(arm.contains(doc.getElementById(req)), `${id} #${req} inside #arm`).toBe(true);
      }
    }
  });
});
