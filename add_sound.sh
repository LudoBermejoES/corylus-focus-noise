#!/bin/bash
# Add a single new ambient sound to the catalog, organized by category.
#
# Usage:
#   export FREESOUND_ACCESS_TOKEN=... (see README for how to get one)
#   ./add_sound.sh <freesound_id> <category> <id> "<Display Name>"
#
# Example:
#   ./add_sound.sh 210540 animals crickets_meadow "Crickets in a Meadow"
#
# This downloads the sound (authenticated), converts it to OGG with metadata
# stripped, places it at sounds_organized/<category>/<id>.ogg, and appends a
# verified entry to VERIFIED_CATALOG.json. Run ./repackage.sh afterwards to
# rebuild the bundle ZIP + manifest for a new release.

set -euo pipefail

VALID_CATEGORIES="animals nature noise places rain things transport urban"

if [ $# -ne 4 ]; then
  echo "Usage: $0 <freesound_id> <category> <id> \"<Display Name>\""
  echo "Categories: $VALID_CATEGORIES"
  exit 1
fi

FS_ID="$1"
CATEGORY="$2"
SOUND_ID="$3"
DISPLAY_NAME="$4"

if [ -z "${FREESOUND_ACCESS_TOKEN:-}" ]; then
  echo "Error: FREESOUND_ACCESS_TOKEN not set (see README.md § Getting a token)"
  exit 1
fi

# Validate category
if ! echo "$VALID_CATEGORIES" | grep -qw "$CATEGORY"; then
  echo "Error: '$CATEGORY' is not a valid category. Use one of: $VALID_CATEGORIES"
  exit 1
fi

ORGANIZED_DIR="sounds_organized"
ogg_file="$ORGANIZED_DIR/$CATEGORY/${SOUND_ID}.ogg"
mkdir -p "$ORGANIZED_DIR/$CATEGORY"

if [ -f "$ogg_file" ]; then
  echo "Error: $ogg_file already exists. Pick a different <id> or delete it first."
  exit 1
fi

# 1. Verify the sound exists and read its real name/license (provenance check)
echo "Verifying freesound id=$FS_ID …"
meta=$(curl -s -H "Authorization: Bearer $FREESOUND_ACCESS_TOKEN" \
  "https://freesound.org/apiv2/sounds/${FS_ID}/?fields=id,name,license,username")
if echo "$meta" | grep -q '"detail"'; then
  echo "Error: freesound id=$FS_ID not found: $meta"
  exit 1
fi
real_name=$(echo "$meta" | python3 -c "import json,sys;print(json.load(sys.stdin)['name'])")
username=$(echo "$meta" | python3 -c "import json,sys;print(json.load(sys.stdin)['username'])")
license_url=$(echo "$meta" | python3 -c "import json,sys;print(json.load(sys.stdin)['license'])")
case "$license_url" in
  *publicdomain/zero*) license="CC0" ;;
  *licenses/by/*)      license="CC-BY 3.0" ;;
  *) echo "Error: license '$license_url' is not CC0 or CC-BY — not redistributable."; exit 1 ;;
esac
echo "  freesound name: \"$real_name\" by $username ($license)"

# 2. Download (with retry) and convert to OGG, stripping metadata
temp_file="/tmp/${SOUND_ID}_raw"
echo "Downloading …"
http_status=000
for attempt in 1 2 3; do
  http_status=$(curl -sL --max-time 120 -w "%{http_code}" \
    -H "Authorization: Bearer $FREESOUND_ACCESS_TOKEN" \
    -o "$temp_file" \
    "https://freesound.org/apiv2/sounds/${FS_ID}/download/")
  [ "$http_status" = "200" ] && break
  echo "  attempt $attempt failed (HTTP $http_status), retrying"; sleep 2
done
[ "$http_status" = "200" ] || { echo "Error: download failed (HTTP $http_status)"; rm -f "$temp_file"; exit 1; }

echo "Converting to OGG-Vorbis …"
# Codec must be explicit: without -c:a, ffmpeg silently substitutes whatever
# default encoder it has for a .ogg extension (e.g. FLAC-in-Ogg when
# libvorbis isn't compiled in) — browsers report the file as "loadable" but
# fail to seek/play/decode it. See design.md's audio-playback spike findings.
vorbis_encoder="libvorbis"
ffmpeg -encoders 2>/dev/null | grep -q " libvorbis " || vorbis_encoder="vorbis -strict -2"
# Force stereo output: the fallback native `vorbis` encoder only supports
# exactly 2 channels and errors out on mono/multi-channel sources.
ffmpeg -i "$temp_file" -c:a $vorbis_encoder -ac 2 -q:a 5 -map_metadata -1 -y "$ogg_file" -loglevel error
rm -f "$temp_file"

actual_codec=$(ffprobe -v error -select_streams a:0 -show_entries stream=codec_name -of csv=p=0 "$ogg_file" 2>/dev/null)
if [ "$actual_codec" != "vorbis" ]; then
  echo "Error: wrong codec after conversion: got '$actual_codec', expected 'vorbis'"
  rm -f "$ogg_file"
  exit 1
fi
echo "  wrote $ogg_file ($(du -h "$ogg_file" | cut -f1), codec=vorbis)"

# 3. Append a verified entry to VERIFIED_CATALOG.json
echo "Updating VERIFIED_CATALOG.json …"
python3 - "$SOUND_ID" "$DISPLAY_NAME" "$CATEGORY" "$FS_ID" "$username" "$license" <<'PY'
import json, sys
sid, name, cat, fsid, author, lic = sys.argv[1:7]
with open("VERIFIED_CATALOG.json") as f:
    data = json.load(f)
if any(s["id"] == sid for s in data["sounds"]):
    print(f"  '{sid}' already in catalog, skipping append"); sys.exit(0)
data["sounds"].append({
    "id": sid, "displayName": name, "category": cat,
    "freesoundId": int(fsid), "author": author, "license": lic,
})
with open("VERIFIED_CATALOG.json", "w") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
print(f"  added '{sid}' ({name})")
PY

echo ""
echo "✓ Done. Next: run ./repackage.sh to rebuild the bundle for a new release."
