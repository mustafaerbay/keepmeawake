// Release helpers used by .github/workflows/release.yml.
//   node scripts/release-notes.mjs notes v0.1.0   -> release body (CHANGELOG section + coffee footer)
//   node scripts/release-notes.mjs check v0.1.0   -> exit 1 if any version file disagrees with the tag
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const COFFEE_FOOTER = `
---
☕ Don'u sevdiysen ona bir kahve ısmarla: https://buymeacoffee.com/mustafaerbay
`;

const strip = (v) => v.replace(/^v/, "");
const esc = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

/** The CHANGELOG section for `version` (without its heading) plus the footer. */
export function notesFor(changelog, version) {
  const v = strip(version);
  const head = new RegExp(`^## \\[${esc(v)}\\][^\\n]*\\n`, "m");
  const m = head.exec(changelog);
  if (!m) throw new Error(`CHANGELOG.md has no section for ${v}`);
  const rest = changelog.slice(m.index + m[0].length);
  const end = rest.search(/^## \[|^\[[^\]]+\]:\s/m);
  const body = (end === -1 ? rest : rest.slice(0, end)).trim();
  return `${body}\n${COFFEE_FOOTER}`;
}

/** Every version file must carry the tag's version. Returns error lines. */
export function checkVersions(tag, { packageJson, cargoToml, tauriConf }) {
  const want = strip(tag);
  const found = {
    "package.json": JSON.parse(packageJson).version,
    "src-tauri/Cargo.toml": (cargoToml.match(/^version\s*=\s*"([^"]+)"/m) || [])[1],
    "src-tauri/tauri.conf.json": JSON.parse(tauriConf).version,
  };
  return Object.entries(found)
    .filter(([, v]) => v !== want)
    .map(([f, v]) => `${f} has version ${v}, tag is ${want}`);
}

const isMain = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  const [cmd, tag] = process.argv.slice(2);
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const read = (f) => fs.readFileSync(path.join(root, f), "utf8");
  if (cmd === "notes") {
    process.stdout.write(notesFor(read("CHANGELOG.md"), tag));
  } else if (cmd === "check") {
    const errs = checkVersions(tag, {
      packageJson: read("package.json"),
      cargoToml: read("src-tauri/Cargo.toml"),
      tauriConf: read("src-tauri/tauri.conf.json"),
    });
    errs.forEach((e) => console.error(e));
    process.exit(errs.length ? 1 : 0);
  } else {
    console.error("usage: release-notes.mjs notes|check <tag>");
    process.exit(2);
  }
}
