#!/usr/bin/env python3
"""Generate assets/relations.ron + assets/impacts.ron from Godot card_db.gd."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GODOT = Path(r"E:\桌面\方寸商国：桃花源记\scripts\cards\card_db.gd")
REL_OUT = ROOT / "assets" / "relations.ron"
IMP_OUT = ROOT / "assets" / "impacts.ron"


def parse_relations(text: str) -> list[dict]:
    rels = []
    for m in re.finditer(r'_rel(?:_near)?\("([^"]+)",\s*"([^"]+)",\s*"([^"]+)"\)', text):
        near = text[m.start() : m.end()].startswith("_rel_near")
        rels.append({"source": m.group(1), "target": m.group(2), "result": m.group(3), "near": near})
    return rels


def parse_impacts(text: str) -> list[dict]:
    impacts = []
    for m in re.finditer(
        r'_impact_reg\(\s*"([^"]*)",\s*"([^"]*)",\s*"([^"]*)"(?:,\s*"([^"]*)")?(?:,\s*Vector2i\((-?\d+),\s*(-?\d+)\))?(?:,\s*"([^"]*)")?\s*\)',
        text,
    ):
        src, tgt, result, extra, ox, oy, handler = m.groups()
        impacts.append(
            {
                "source": src or "*",
                "target": tgt,
                "result": result or "",
                "extra_result": extra or "",
                "extra_offset": (int(ox or 0), int(oy or 0)),
                "handler_id": handler or "",
                "hits_required": 2,
            }
        )
    return impacts


def emit_relations(rels: list[dict]) -> str:
    lines = ["["]
    for r in rels:
        near = ", near: true" if r.get("near") else ""
        lines.append(f'    (source: "{r["source"]}", target: "{r["target"]}", result: "{r["result"]}"{near}),')
    lines.append("]")
    return "\n".join(lines) + "\n"


def emit_impacts(imps: list[dict]) -> str:
    lines = ["["]
    for r in imps:
        parts = [
            f'source: "{r["source"]}"',
            f'target: "{r["target"]}"',
        ]
        if r["result"]:
            parts.append(f'result: "{r["result"]}"')
        if r.get("extra_result"):
            parts.append(f'extra_result: "{r["extra_result"]}"')
        ox, oy = r.get("extra_offset", (0, 0))
        if (ox, oy) != (0, 0):
            parts.append(f"extra_offset: ({ox}, {oy})")
        if r.get("handler_id"):
            parts.append(f'handler_id: "{r["handler_id"]}"')
        parts.append("hits_required: 2")
        lines.append(f"    ({', '.join(parts)}),")
    lines.append("]")
    return "\n".join(lines) + "\n"


def main() -> None:
    text = GODOT.read_text(encoding="utf-8")
    REL_OUT.write_text(emit_relations(parse_relations(text)), encoding="utf-8")
    IMP_OUT.write_text(emit_impacts(parse_impacts(text)), encoding="utf-8")
    print(f"Wrote {REL_OUT} and {IMP_OUT}")


if __name__ == "__main__":
    main()
