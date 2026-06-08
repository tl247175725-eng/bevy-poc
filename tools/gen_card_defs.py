#!/usr/bin/env python3
"""Generate assets/card_defs.ron from Godot card_db.gd + card_base.gd colors."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GODOT = Path(r"E:\桌面\方寸商国：桃花源记\scripts\cards")
CARD_DB = GODOT / "card_db.gd"
CARD_BASE = GODOT / "card_base.gd"
OUT = ROOT / "assets" / "card_defs.ron"

DEFAULT_BG = "fffdf5"


def parse_colors(text: str) -> dict[str, str]:
    m = re.search(r'var styles = \{([^}]+(?:\{[^}]*\}[^}]*)*)\}', text, re.DOTALL)
    if not m:
        raise RuntimeError("styles dict not found")
    block = m.group(1)
    colors: dict[str, str] = {}
    for line in block.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        mm = re.match(r'"([^"]+)":\s*\["([0-9a-fA-F]+)"', line)
        if mm:
            colors[mm.group(1)] = mm.group(2)
    return colors


def hex_to_rgba(h: str) -> tuple[int, int, int, int]:
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), 255)


def parse_regs(text: str) -> list[dict]:
    cards = []
    for m in re.finditer(
        r'_reg\("([^"]+)",\s*"([^"]+)",\s*"([^"]+)",\s*"[^"]+",\s*\[([^\]]*)\](?:,\s*(\d+))?(?:,\s*(true|false))?',
        text,
    ):
        type_name, icon, display, tags_raw, hp, rooted = m.groups()
        tags = re.findall(r'"([^"]+)"', tags_raw)
        cards.append(
            {
                "type_name": type_name,
                "icon": icon,
                "display_name": display,
                "tags": tags,
                "hp": int(hp) if hp else 0,
                "is_rooted": rooted == "true",
            }
        )
    return cards


def ron_escape(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


def main() -> None:
    db_text = CARD_DB.read_text(encoding="utf-8")
    base_text = CARD_BASE.read_text(encoding="utf-8")
    colors = parse_colors(base_text)
    cards = parse_regs(db_text)
    if len(cards) < 50:
        raise RuntimeError(f"expected 50+ cards, got {len(cards)}")

    lines = ["["]
    for c in cards:
        bg = colors.get(c["type_name"], DEFAULT_BG)
        r, g, b, a = hex_to_rgba(bg)
        tag_items = ", ".join(f'"{t}"' for t in c["tags"])
        lines.append("    CardDef(")
        lines.append(f'        type_name: "{ron_escape(c["type_name"])}",')
        lines.append(f'        display_name: "{ron_escape(c["display_name"])}",')
        lines.append(f'        icon: "{ron_escape(c["icon"])}",')
        lines.append(f"        tags: [{tag_items}],")
        lines.append(f"        color: ({r}, {g}, {b}, {a}),")
        lines.append(f"        hp: {c['hp']},")
        lines.append(f"        is_rooted: {'true' if c['is_rooted'] else 'false'},")
        lines.append("    ),")
    lines.append("]")
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(cards)} cards to {OUT}")


if __name__ == "__main__":
    main()
