#!/bin/bash
# Download VERIFIED ambient sounds from freesound.org and convert to OGG format
# Requires FREESOUND_ACCESS_TOKEN env var (OAuth2 bearer token, expires in 24h)
# All sound IDs below were verified against the freesound.org search API:
# name/license/author confirmed to match before inclusion (see VERIFIED_CATALOG.json)

if [ -z "$FREESOUND_ACCESS_TOKEN" ]; then
  echo "Error: FREESOUND_ACCESS_TOKEN environment variable not set"
  exit 1
fi

ORGANIZED_DIR="sounds_organized"
TEMP_DIR="/tmp/freesound_downloads"

mkdir -p "$TEMP_DIR"
for cat in animals nature noise places rain things transport urban; do
  mkdir -p "$ORGANIZED_DIR/$cat"
done

# Array of (id, freesound_sound_id, category, name) - all verified
declare -a SOUNDS=(
  "river|660265|nature|River"
  "campfire|588401|nature|Campfire"
  "wind_forest|667597|nature|Wind Through Trees"
  "ocean_waves|803679|nature|Ocean Waves"
  "waterfall|442475|nature|Waterfall"
  "forest_ambience|523372|nature|Forest Ambience"
  "thunder_distant|581232|nature|Distant Thunder"

  "light_rain|595717|rain|Light Rain"
  "heavy_rain|570307|rain|Heavy Rain"
  "rain_on_window|236292|rain|Rain on Window"
  "rain_on_roof|521772|rain|Rain on Tin Roof"
  "rain_in_forest|641871|rain|Rain in Forest"
  "rain_thunder|810880|rain|Rain with Thunder"

  "birds_chirping|852999|animals|Birds Chirping"
  "crickets_night|746365|animals|Crickets at Night"
  "seagulls|533042|animals|Seagulls"
  "owl_hoot|745208|animals|Owl Hoot"
  "cat_purring|149487|animals|Cat Purring"
  "farm_night|355339|animals|Farm at Night"

  "city_ambience|561463|urban|City Ambience"
  "busy_street|238718|urban|Busy Street"
  "distant_siren|469363|urban|Distant Siren"
  "construction_site|479535|urban|Construction Site"
  "highway_traffic|453584|urban|Highway Traffic"
  "distant_fireworks|434224|urban|Distant Fireworks"

  "coffee_shop|370973|places|Coffee Shop"
  "library_quiet|635727|places|Quiet Library"
  "office_typing|636861|places|Office with Typing"
  "restaurant_diner|627079|places|Restaurant Diner"
  "subway_station|451720|places|Subway Station"
  "airport_terminal|210786|places|Airport Terminal"

  "train_interior|341208|transport|Train Interior"
  "airplane_cabin|456092|transport|Airplane Cabin"
  "helicopter_flyby|546805|transport|Helicopter Flyby"
  "car_interior|397115|transport|Car Interior Driving"
  "rowing_boat|588307|transport|Rowing Boat"
  "kayaking|509323|transport|Kayaking Calm Water"

  "clock_ticking|417593|things|Clock Ticking"
  "typewriter|801119|things|Typewriter Typing"
  "keyboard_typing|638035|things|Mechanical Keyboard"
  "page_turn|336374|things|Page Turning"
  "fan_ambience|269594|things|Fan Ambience"
  "computer_noises|439627|things|Computer Noises"

  "white_noise|249313|noise|White Noise"
  "pink_noise|470754|noise|Pink Noise"
  "brown_noise|242513|noise|Brown Noise"
  "soft_brown_noise|853302|noise|Soft Brownian Noise"
  "pink_noise_filtered|637144|noise|Filtered Pink Noise"
  "air_tone_noise|437281|noise|Air Tone Ambience"
)

echo "Downloading and converting ${#SOUNDS[@]} verified ambient sounds to OGG format..."
echo ""

count=0
failed=0
for sound_data in "${SOUNDS[@]}"; do
  IFS='|' read -r id sound_id category name <<< "$sound_data"

  count=$((count + 1))
  echo "[$count/${#SOUNDS[@]}] $name → $category/"

  temp_file="$TEMP_DIR/${id}_raw"
  ogg_file="$ORGANIZED_DIR/$category/${id}.ogg"

  # Resume: skip if already converted
  if [ -f "$ogg_file" ]; then
    echo "  ↷ Already done, skipping"
    continue
  fi

  # Download with OAuth2 bearer token, retrying on transient failure
  http_status=000
  for attempt in 1 2 3; do
    http_status=$(curl -sL --max-time 120 -w "%{http_code}" \
      -H "Authorization: Bearer $FREESOUND_ACCESS_TOKEN" \
      -o "$temp_file" \
      "https://freesound.org/apiv2/sounds/${sound_id}/download/")
    [ "$http_status" = "200" ] && break
    echo "  … attempt $attempt failed (HTTP $http_status), retrying"
    sleep 2
  done

  if [ "$http_status" != "200" ]; then
    echo "  ✗ Download failed (HTTP $http_status)"
    failed=$((failed + 1))
    rm -f "$temp_file"
    continue
  fi

  # Verify it's actually audio, not an error page
  file_type=$(file -b "$temp_file")
  if [[ "$file_type" != *"WAVE"* ]] && [[ "$file_type" != *"MPEG"* ]] && [[ "$file_type" != *"Ogg"* ]] && [[ "$file_type" != *"FLAC"* ]] && [[ "$file_type" != *"AIFF"* ]] && [[ "$file_type" != *"ISO Media"* ]] && [[ "$file_type" != *"MP4"* ]] && [[ "$file_type" != *"M4A"* ]]; then
    echo "  ✗ Not valid audio: $file_type"
    failed=$((failed + 1))
    rm -f "$temp_file"
    continue
  fi

  # Convert to OGG, stripping all metadata
  if ffmpeg -i "$temp_file" -q:a 5 -map_metadata -1 -y "$ogg_file" -loglevel error; then
    echo "  ✓ Converted ($(du -h "$ogg_file" | cut -f1))"
  else
    echo "  ✗ ffmpeg conversion failed"
    failed=$((failed + 1))
  fi

  rm -f "$temp_file"
done

echo ""
echo "✓ Successfully processed $((count - failed))/${count} files"
if [ "$failed" -gt 0 ]; then
  echo "✗ $failed files failed"
fi
