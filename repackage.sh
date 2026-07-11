#!/bin/bash
# Rebuild the ambient-sounds bundle ZIP + BUNDLE_MANIFEST.json from the
# category-organized OGG files, and optionally publish a new GitHub Release
# with both the ZIP and the loose per-sound OGGs.
#
# Usage:
#   ./repackage.sh <version>            # rebuild ZIP + manifest only
#   ./repackage.sh <version> --publish  # also create/upload the GitHub Release
#
# Example:
#   ./repackage.sh 1.1.0 --publish
#
# The source of truth is sounds_organized/<category>/<id>.ogg. Add new sounds
# with ./add_sound.sh (which keeps that structure), then run this.

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <version> [--publish]"
  exit 1
fi

VERSION="$1"
PUBLISH="${2:-}"
ORGANIZED_DIR="sounds_organized"
ZIP="ambient-sounds-bundle.zip"
TAG="sounds-v${VERSION}"

if [ ! -d "$ORGANIZED_DIR" ]; then
  echo "Error: $ORGANIZED_DIR not found. Run download_and_convert.sh or add_sound.sh first."
  exit 1
fi

# 1. Build the ZIP from the category folders (exclude dotfiles like .DS_Store),
#    plus names.json (localized sound/preset display names) at the ZIP root.
echo "Building $ZIP from $ORGANIZED_DIR/ + names.json …"
if [ ! -f "names.json" ]; then
  echo "Error: names.json not found at repo root. It ships inside the bundle" \
       "alongside the audio so translations don't require a Rust code change."
  exit 1
fi
rm -f "$ZIP"
( cd "$ORGANIZED_DIR" && zip -r -q "../$ZIP" . -x ".*" -x "*/.*" )
zip -q "$ZIP" names.json

SHA=$(shasum -a 256 "$ZIP" | cut -d' ' -f1)
SIZE=$(stat -f%z "$ZIP" 2>/dev/null || stat -c%s "$ZIP")
echo "  $ZIP: $((SIZE/1000000))MB  sha256=$SHA"

# 2. Regenerate BUNDLE_MANIFEST.json from VERIFIED_CATALOG.json + the ZIP facts
echo "Writing BUNDLE_MANIFEST.json …"
python3 - "$VERSION" "$SHA" "$SIZE" "$ZIP" <<'PY'
import json, sys, os
version, sha, size, zipname = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
with open("VERIFIED_CATALOG.json") as f:
    cat = json.load(f)
sounds = cat["sounds"]

# Cross-check: every catalog entry must have a file on disk, and vice-versa.
on_disk = set()
for root, _, files in os.walk("sounds_organized"):
    for fn in files:
        if fn.endswith(".ogg"):
            c = os.path.basename(root)
            on_disk.add(f"{c}/{fn[:-4]}")
in_catalog = {f"{s['category']}/{s['id']}" for s in sounds}
missing_files = in_catalog - on_disk
orphan_files = on_disk - in_catalog
if missing_files:
    print("  ERROR: catalog entries with no OGG file:", sorted(missing_files)); sys.exit(1)
if orphan_files:
    print("  WARNING: OGG files not in catalog (won't be documented):", sorted(orphan_files))

# Cross-check: every catalog sound/preset id must have a name in every
# language names.json ships, so a translation gap fails the build instead of
# silently falling back to English (or the raw id) at runtime.
with open("names.json") as f:
    names = json.load(f)
with open("PRESETS.json") as f:
    presets = json.load(f)["presets"]

sound_ids = {s["id"] for s in sounds}
preset_ids = {p["id"] for p in presets}
for lang, section in names.get("sounds", {}).items():
    missing = sound_ids - set(section.keys())
    if missing:
        print(f"  ERROR: names.json sounds.{lang} missing translations for:", sorted(missing)); sys.exit(1)
for lang, section in names.get("presets", {}).items():
    missing = preset_ids - set(section.keys())
    if missing:
        print(f"  ERROR: names.json presets.{lang} missing translations for:", sorted(missing)); sys.exit(1)

manifest = {
    "bundleVersion": version,
    "bundleFile": zipname,
    "bundleSha256": sha,
    "bundleSizeBytes": size,
    "soundCount": len(sounds),
    "categories": sorted({s["category"] for s in sounds}),
    "layout": "<category>/<id>.ogg",
    "note": "Bundle published as a GitHub Release asset on corylus-focus-noise. "
            "Not committed to git. Regenerate with download_and_convert.sh / add_sound.sh + repackage.sh.",
    "sounds": sounds,
}
with open("BUNDLE_MANIFEST.json", "w") as f:
    json.dump(manifest, f, indent=2); f.write("\n")

# per-category counts sanity print
from collections import Counter
counts = Counter(s["category"] for s in sounds)
for c in sorted(counts):
    flag = "" if counts[c] >= 6 else "  ⚠ under 6"
    print(f"    {c}: {counts[c]}{flag}")
print(f"  {len(sounds)} sounds total")
PY

if [ "$PUBLISH" != "--publish" ]; then
  echo ""
  echo "✓ Rebuilt $ZIP + BUNDLE_MANIFEST.json (version $VERSION)."
  echo "  Re-run with --publish to create GitHub Release $TAG."
  echo "  Remember to commit BUNDLE_MANIFEST.json + VERIFIED_CATALOG.json."
  exit 0
fi

# 3. Publish the GitHub Release: ZIP + loose per-sound OGGs (category-prefixed names)
echo "Publishing GitHub Release $TAG …"
ASSET_DIR="/tmp/rel_assets_${VERSION}"
rm -rf "$ASSET_DIR"; mkdir -p "$ASSET_DIR"
for f in "$ORGANIZED_DIR"/*/*.ogg; do
  c=$(basename "$(dirname "$f")")
  cp "$f" "$ASSET_DIR/${c}__$(basename "$f")"
done

gh release create "$TAG" "$ZIP" "$ASSET_DIR"/*.ogg \
  --title "Ambient Sounds Bundle v${VERSION}" \
  --notes "$(python3 -c "import json;m=json.load(open('BUNDLE_MANIFEST.json'));print(f\"{m['soundCount']} CC0/CC-BY ambient sounds (OGG Vorbis) across {len(m['categories'])} categories.\n\nSHA-256: \`{m['bundleSha256']}\`\nSize: {m['bundleSizeBytes']//1000000}MB · Layout: \`{m['layout']}\`\n\nRuntime download target for the Corylus ambient-sounds module. Loose per-sound OGGs are attached as <category>__<id>.ogg. See BUNDLE_MANIFEST.json for the full list and attribution.\")")"

echo ""
echo "✓ Published $TAG with the ZIP + $(ls "$ASSET_DIR" | wc -l | tr -d ' ') loose OGGs."
echo "  Now commit BUNDLE_MANIFEST.json + VERIFIED_CATALOG.json and push."
