#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
icon_tmp=$(mktemp -d)
trap 'rm -rf "$icon_tmp"' EXIT
./desktop/node_modules/.bin/tauri icon desktop/assets/app-icon.svg --output "$icon_tmp"
for name in 32x32.png 128x128.png icon.icns icon.ico; do
  cp "$icon_tmp/$name" "desktop/src-tauri/icons/$name"
done
./desktop/node_modules/.bin/tauri icon desktop/assets/app-icon.svg --output "$icon_tmp/sizes" --png 512 --png 1024
cp "$icon_tmp/sizes/512x512.png" desktop/src-tauri/icons/icon.png
cp "$icon_tmp/sizes/1024x1024.png" desktop/assets/app-icon-master.png
cp desktop/src-tauri/icons/128x128.png desktop/src/assets/app-icon.png
