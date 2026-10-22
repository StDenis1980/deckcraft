#!/usr/bin/env bash
# Regenerate every app icon: the IberOffice azulejo tile, drawn in code by
# packaging/iberoffice-icons.py (deckcraft.svg, the hicolor PNGs, the .ico and the .icns).
#
# Needs python3 with cairosvg and Pillow. The outputs are committed, so builds and packaging never
# need these tools. Update the app-icon rows in ATTRIBUTION.md if the files change.
#
#   packaging/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
python3 "$ROOT/packaging/iberoffice-icons.py" apply "$ROOT" presenta deckcraft
echo "icons written to $ROOT/assets/app-icon"
