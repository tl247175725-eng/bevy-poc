import re
from pathlib import Path

text = Path("assets/card_defs.ron").read_text(encoding="utf-8")
blocks = re.findall(r"tags: \[(.*?)\]", text, re.S)
tags = set()
for block in blocks:
    tags.update(re.findall(r'"([^"]+)"', block))
for t in sorted(tags):
    print(t)
