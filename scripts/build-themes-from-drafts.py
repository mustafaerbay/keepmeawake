"""One-off: turn design/drafts/don-*.svg into themes/<id>/character.svg with the
theme contract ids (lids, flame, wisps at the tip, exhale puffs at the mouth, zzz).
Kept in the repo so the transformation is reproducible."""
import json, re, pathlib
R = pathlib.Path(__file__).resolve().parent.parent
THEMES = [
  # id, draft, name, order, tip, mouth, eyes, eye_r, skin, shoulder, neck
  ("don-tombul", "a", "Tombul Don", 1, (152,126), (110,132), [(87,108),(113,108)], 6, "#f2c9a0", (140,166), (100,150)),
  ("don-sinsi",  "b", "Sinsi Don",  2, (150,119), (106,131), [(89,105),(111,105)], 6, "#e9bf95", (130,170), (100,142)),
  ("don-sert",   "c", "Sert Don",   3, (158,131), (110,136), [(86,108),(114,108)], 7, "#e6b78c", (150,172), (100,150)),
]
for tid, d, name, order, tip, mouth, eyes, er, skin, shoulder, neck in THEMES:
    s = (R/f"design/drafts/don-{d}.svg").read_text()
    s = re.sub(r'\s*<!--.*?-->', '', s)
    # drop the static smoke group from the draft
    s = re.sub(r'\s*<g fill="#d4d4dc".*?</g>\n', '\n', s, flags=re.S)
    tx, ty = tip; mx, my = mouth
    flame = (f'\n        <g id="wisps" stroke="none" fill="#b9b9c6">'
             f'<g id="flame" opacity="0"><path d="M{tx+1} {ty-2} Q{tx-6} {ty-14} {tx+1} {ty-24} Q{tx+8} {ty-14} {tx+1} {ty-2} Z" '
             f'fill="#ffb21f" stroke="#d9480f" stroke-width="2"/></g>'
             + ''.join(f'<circle id="smoke-{i}" class="wisp" cx="{tx+1}" cy="{ty-6}" r="5"/>' for i in (1,2,3))
             + '</g>')
    # insert right after the #cigar group closes (inside #arm)
    ci = s.index('<g id="cigar">'); ce = s.index('</g>', ci) + len('</g>')
    s = s[:ce] + flame + s[ce:]
    lids = ''.join(f'<ellipse cx="{x}" cy="{y}" rx="{er}" ry="{er-1}" fill="{skin}" stroke="none"/>'
                   f'<path d="M{x-er} {y} Q{x} {y+4} {x+er} {y}" fill="none" stroke-width="3"/>' for x, y in eyes)
    zzz = f'<text id="zzz" x="{neck[0]+44}" y="{neck[1]-110}" font-family="Georgia,serif" font-weight="700" font-size="22" fill="#5b6cff" stroke="none" opacity="0">z</text>'
    s = s.replace('<g id="arm">', f'<g id="lids" opacity="0">{lids}</g>\n    </g>\n    <g id="arm">', 1)
    # the draft's head group closed right before arm; remove that now-duplicate close
    s = s.replace('    </g>\n    <g id="lids"', '    <g id="lids"', 1)
    puffs = ''.join(f'<circle id="smoke-{i}" class="puff" cx="{mx}" cy="{my}" r="6"/>' for i in (4,5))
    s = s.replace('  </g>\n</svg>', f'    <g id="puffs" stroke="none" fill="#b9b9c6">{puffs}</g>\n    {zzz}\n  </g>\n</svg>')
    out = R/"themes"/tid; out.mkdir(parents=True, exist_ok=True)
    (out/"character.svg").write_text(s)
    (out/"theme.json").write_text(json.dumps({
        "id": tid, "name": name, "order": order, "window": {"width": 200, "height": 260},
        "tray": {"fps": 5, "awakeFrames": 6, "viewBox": "52 30 120 120"},
        "transitions": {"lightingMs": 1200, "snuffingMs": 1000}}, indent=2) + "\n")
    (out/"character.css").write_text(
        f"/* {name}: pivots for the shared smoker animation (themes/_shared/smoker.css) */\n"
        f".don-root {{\n  --shoulder: {shoulder[0]}px {shoulder[1]}px;\n  --neck: {neck[0]}px {neck[1]}px;\n"
        f"  --tip: {tip[0]+1}px {tip[1]-6}px;\n"
        f"  --rest-angle: 55deg;\n  --sleep-angle: 95deg;\n}}\n")
    print("wrote", tid)
