#!/usr/bin/env python3
"""Package the public design preview from an explicit file allowlist."""
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
out = root / 'output' / 'website-preview'
out.mkdir(parents=True, exist_ok=True)
for name in ('index.html', 'site.css', 'site.js', 'live-data.mjs'):
    shutil.copy2(root / 'docs' / 'designs' / 'website' / name, out / name)
(out / 'assets').mkdir(exist_ok=True)
for source, name in (
    ('docs/assets/readme/library.png', 'library.png'),
    ('desktop/src-tauri/icons/128x128.png', 'icon.png'),
):
    shutil.copy2(root / source, out / 'assets' / name)
print(out)

# Reuse client sources, classification and variable parsing instead of drifting copies.
import json
(out / 'data').mkdir(exist_ok=True)
(out / 'shared').mkdir(exist_ok=True)
sources = root / 'desktop/src/data/skill-sources.json'
shutil.copy2(sources, out / 'data/skill-sources.json')
classification = (root / 'desktop/src/platform/skillCategories.js').read_text()
classification = classification.replace("import sources from '../data/skill-sources.json';", "const sources = " + json.dumps(json.loads(sources.read_text()), ensure_ascii=False) + ";")
(out / 'shared/skillCategories.mjs').write_text(classification)
shutil.copy2(root / 'desktop/src/lib/renderPrompt.js', out / 'shared/renderPrompt.mjs')
shutil.copy2(root / 'docs/assets/readme/square.png', out / 'assets/square.png')
