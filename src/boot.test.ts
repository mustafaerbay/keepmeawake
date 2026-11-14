import { describe, it, expect } from "vitest";
import { boot, type BootDeps } from "./boot";

function fakeDeps(log: string[], handlers: Record<string, (p: unknown) => void>): BootDeps {
  return {
    listen: async (name, cb) => { log.push(`listen:${name}`); handlers[name] = cb as (p: unknown) => void; },
    getState: async () => { log.push("getState"); return { awake: true, displayMaySleep: false, clickThrough: false }; },
    getTheme: async () => { log.push("getTheme"); return "don-tombul"; },
    isVisible: async () => { log.push("isVisible"); return true; },
    onState: (s) => log.push(`state:${s.awake}`),
    onVisible: (v) => log.push(`visible:${v}`),
    onTheme: async (id) => { log.push(`theme:${id}`); },
  };
}

describe("boot", () => {
  it("subscribes_before_reading_initial_state", async () => {
    const log: string[] = [];
    await boot(fakeDeps(log, {}));
    const firstRead = log.findIndex((l) => l === "getState" || l === "getTheme" || l === "isVisible");
    for (const ev of ["state-changed", "don-visibility", "theme-changed"]) {
      const i = log.indexOf(`listen:${ev}`);
      expect(i, ev).toBeGreaterThanOrEqual(0);
      expect(i, ev).toBeLessThan(firstRead);
    }
  });

  it("visibility_comes_from_the_window_not_document_hidden", async () => {
    const log: string[] = [];
    await boot(fakeDeps(log, {}));
    expect(log).toContain("visible:true");
  });

  it("event_during_startup_is_not_overwritten_by_stale_initial_state", async () => {
    const log: string[] = [];
    const handlers: Record<string, (p: unknown) => void> = {};
    const deps = fakeDeps(log, handlers);
    deps.getState = async () => {
      // a toggle lands while the initial read is in flight
      handlers["state-changed"]({ state: { awake: false, displayMaySleep: false, clickThrough: false }, transition: "snuffing" });
      return { awake: true, displayMaySleep: false, clickThrough: false };
    };
    await boot(deps);
    expect(log.filter((l) => l.startsWith("state:")).slice(-1)[0]).toBe("state:false");
  });
});
