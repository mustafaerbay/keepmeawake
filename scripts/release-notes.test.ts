import { describe, it, expect } from "vitest";
// @ts-expect-error plain ESM script without types
import { notesFor, checkVersions, COFFEE_FOOTER, INSTALL_NOTE } from "./release-notes.mjs";

const CHANGELOG = `# Değişiklik günlüğü

## [Unreleased]

- henüz çıkmadı

## [0.2.0] - 2026-12-01

### Eklendi
- yeni şey

## [0.1.0] - 2026-11-15

### Eklendi
- ilk sürüm

[0.2.0]: https://example.com/0.2.0
`;

describe("notesFor", () => {
  it("returns_only_that_versions_section_plus_coffee_footer", () => {
    const n = notesFor(CHANGELOG, "0.2.0");
    expect(n).toContain("### Eklendi\n- yeni şey");
    expect(n).not.toContain("ilk sürüm");
    expect(n).not.toContain("henüz çıkmadı");
    expect(n.trimEnd().endsWith(COFFEE_FOOTER.trim())).toBe(true);
  });

  it("last_section_stops_before_link_references", () => {
    const n = notesFor(CHANGELOG, "0.1.0");
    expect(n).toContain("- ilk sürüm");
    expect(n).not.toContain("https://example.com/0.2.0");
  });

  it("accepts_a_v_prefixed_tag", () => {
    expect(notesFor(CHANGELOG, "v0.1.0")).toContain("- ilk sürüm");
  });

  it("missing_version_throws", () => {
    expect(() => notesFor(CHANGELOG, "9.9.9")).toThrow(/9\.9\.9/);
  });

  it("every_release_explains_the_unsigned_app_warnings", () => {
    const n = notesFor(CHANGELOG, "0.1.0");
    expect(n).toContain(INSTALL_NOTE.trim());
    expect(INSTALL_NOTE).toContain("Apple could not verify");
    expect(INSTALL_NOTE).toContain("Gizlilik ve Güvenlik");
    expect(INSTALL_NOTE).toContain("xattr -dr com.apple.quarantine /Applications/keepmeawake.app");
    expect(INSTALL_NOTE).toContain("SmartScreen");
    // install help comes after the changes, coffee stays last
    expect(n.indexOf("- ilk sürüm")).toBeLessThan(n.indexOf(INSTALL_NOTE.trim()));
    expect(n.indexOf(INSTALL_NOTE.trim())).toBeLessThan(n.indexOf(COFFEE_FOOTER.trim()));
  });

  it("footer_links_buy_me_a_coffee", () => {
    expect(COFFEE_FOOTER).toContain("https://buymeacoffee.com/mustafaerbay");
  });
});

describe("checkVersions", () => {
  const files = { packageJson: '{"version":"0.1.0"}', cargoToml: 'name = "x"\nversion = "0.1.0"\n', tauriConf: '{"version":"0.1.0"}' };

  it("all_match_returns_no_errors", () => {
    expect(checkVersions("v0.1.0", files)).toEqual([]);
  });

  it("reports_each_mismatch", () => {
    const errs = checkVersions("v0.2.0", { ...files, cargoToml: 'version = "0.2.0"\n' });
    expect(errs).toHaveLength(2);
    expect(errs.join("\n")).toMatch(/package\.json/);
    expect(errs.join("\n")).toMatch(/tauri\.conf\.json/);
  });
});
