import type { Transition } from "./theme";

export interface DonState { awake: boolean; displayMaySleep: boolean; clickThrough: boolean }

export interface BootDeps {
  listen<T>(event: string, cb: (payload: T) => void): Promise<unknown>;
  getState(): Promise<DonState>;
  getTheme(): Promise<string>;
  isVisible(): Promise<boolean>;
  onState(s: DonState, transition: Transition): void;
  onVisible(visible: boolean): void;
  onTheme(id: string): Promise<void>;
}

/** Subscribe first, then read: an event that lands while we read the
 *  initial values must win over those (by then stale) values. */
export async function boot(d: BootDeps): Promise<void> {
  let stateSeen = false;
  let themeSeen = false;
  let visibleSeen = false;
  await d.listen<{ state: DonState; transition: Transition }>("state-changed", (p) => {
    stateSeen = true;
    d.onState(p.state, p.transition);
  });
  await d.listen<{ visible: boolean }>("don-visibility", (p) => {
    visibleSeen = true;
    d.onVisible(p.visible);
  });
  await d.listen<{ id: string }>("theme-changed", (p) => {
    themeSeen = true;
    void d.onTheme(p.id);
  });
  const [state, theme, visible] = await Promise.all([d.getState(), d.getTheme(), d.isVisible()]);
  if (!themeSeen) await d.onTheme(theme);
  if (!stateSeen) d.onState(state, null);
  if (!visibleSeen) d.onVisible(visible);
}
