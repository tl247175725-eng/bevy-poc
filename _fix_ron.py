import re

with open('assets/tags.ron', 'r', encoding='utf-8') as f:
    content = f.read()

lines = content.split('\n')
result = []
for line in lines:
    # Match indented identifier keys: spaces + identifier + : + space + {
    m = re.match(r'^(\s+)([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*)(:)\s*(\{.*)$', line)
    if m:
        indent, key, colon, rest = m.groups()
        if not key.startswith('"'):
            line = f'{indent}"{key}"{colon} {rest}'
    result.append(line)

with open('assets/tags.ron', 'w', encoding='utf-8') as f:
    f.write('\n'.join(result))
print('Done')
