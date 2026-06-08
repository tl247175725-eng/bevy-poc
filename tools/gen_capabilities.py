#!/usr/bin/env python3
"""Generate src/capabilities.rs from Godot world_rules.gd CARD_CAPABILITIES."""
import re
from pathlib import Path

SRC = Path(r"E:\桌面\方寸商国：桃花源记\scripts\core\world_rules.gd")
OUT = Path(__file__).resolve().parent.parent / "src" / "capabilities.rs"

def main() -> None:
    text = SRC.read_text(encoding="utf-8")
    m = re.search(r"const CARD_CAPABILITIES := \{([^}]+)\}", text, re.DOTALL)
    if not m:
        raise RuntimeError("CARD_CAPABILITIES not found")
    block = m.group(1)
    entries: list[tuple[str, list[str]]] = []
    for mm in re.finditer(r'"([^"]+)":\s*\[([^\]]*)\]', block):
        name = mm.group(1)
        caps = re.findall(r'"([^"]+)"', mm.group(2))
        entries.append((name, caps))

    lines = [
        "use std::collections::HashMap;",
        "use std::sync::LazyLock;",
        "",
        "static CAPABILITIES: LazyLock<HashMap<&'static str, &'static [&'static str]>> = LazyLock::new(|| {",
        "    let mut m = HashMap::new();",
    ]
    for name, caps in entries:
        if caps:
            cap_list = ", ".join(f'"{c}"' for c in caps)
            lines.append(f'    m.insert("{name}", &[{cap_list}][..]);')
        else:
            lines.append(f'    m.insert("{name}", &[][..]);')
    lines.extend(
        [
            "    m",
            "});",
            "",
            "pub fn card_capabilities(type_name: &str) -> &'static [&'static str] {",
            "    CAPABILITIES.get(type_name).copied().unwrap_or(&[])",
            "}",
            "",
            "pub fn all_capability_cards() -> impl Iterator<Item = &'static str> {",
            "    CAPABILITIES.keys().copied()",
            "}",
            "",
            "pub fn capability_count() -> usize {",
            "    CAPABILITIES.len()",
            "}",
            "",
        ]
    )
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(entries)} capability entries to {OUT}")


if __name__ == "__main__":
    main()
