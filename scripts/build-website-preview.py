#!/usr/bin/env python3
"""Package the public design preview from an explicit file allowlist."""
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
out = root / 'output' / 'website-preview'
out.mkdir(parents=True, exist_ok=True)
for name in ('index.html', 'site.css', 'site.js'):
    shutil.copy2(root / 'docs' / 'designs' / 'website' / name, out / name)
(out / 'assets').mkdir(exist_ok=True)
for source, name in (
    ('docs/assets/readme/library.png', 'library.png'),
    ('desktop/src-tauri/icons/128x128.png', 'icon.png'),
):
    shutil.copy2(root / source, out / 'assets' / name)
print(out)
