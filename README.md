# corylus-focus-noise

Ambient sound catalog and build pipeline for the [Corylus](https://github.com/LudoBermejoES/corylus)
writing app's **Ambient Sounds** module (an "A Soft Murmur"-style
background-noise mixer).

Vendored as a git submodule at `src-tauri/vendor/corylus-focus-noise` in the
main Corylus repo. This repo holds the **source-of-truth text** (catalog,
scripts, manifest) and the Rust engine crate. The audio files themselves are
**not** committed to git — they are published as **GitHub Release assets** and
downloaded by the app at runtime as a single bundle. See
[Why the audio isn't in git](#why-the-audio-isnt-in-git).

## What's in the repo

| File | Purpose |
|---|---|
| `VERIFIED_CATALOG.json` | The catalog: every sound's `id`, display name, category, freesound id, author, license. Source of truth for what exists. |
| `BUNDLE_MANIFEST.json` | Records the currently-published bundle: version, ZIP SHA-256, size, full sound list. The app verifies its download against this. |
| `download_and_convert.sh` | Build-time: fetch the **whole** catalog from freesound, convert to OGG, lay out by category. |
| `add_sound.sh` | Build-time: add **one** new sound to the catalog + category folder. |
| `repackage.sh` | Rebuild the bundle ZIP + manifest from the category folders, optionally publish a new Release. |
| `src/` | The Rust engine crate (module install/state, mix save/recall). |

Audio lives at `sounds_organized/<category>/<id>.ogg` locally, but that
directory is **gitignored** — regenerate it with the scripts below.

## The 8 categories

`animals`, `nature`, `noise`, `places`, `rain`, `things`, `transport`, `urban`
— minimum 6 sounds each.

## Where the audio actually is

Not in the code tree — on the **[Releases](../../releases)** page:

- `ambient-sounds-bundle.zip` — all sounds in one archive (`<category>/<id>.ogg`).
  This is what the app downloads and unzips on install (all-at-once).
- Loose per-sound OGGs, named `<category>__<id>.ogg`, for previewing or
  swapping individual sounds without touching the ZIP.

## Getting a freesound token (needed to download audio)

freesound.org requires OAuth2 to download files, so the build scripts need a
short-lived access token (valid 24h):

1. Create a free freesound.org account and an API app at
   <https://freesound.org/apiv2/apply/>. For a non-web app's callback URL use
   `http://freesound.org/home/app_permissions/permission_granted/`. Note the
   **client id** and **client secret / api key**.
2. Open this in a browser (replace `CLIENT_ID`):
   `https://freesound.org/apiv2/oauth2/authorize/?client_id=CLIENT_ID&response_type=code`
   Log in, approve, copy the `code=…` value from the redirected URL (the page
   fails to load — that's fine, you only need the code).
3. Exchange the code for a token:
   ```bash
   curl -s -X POST https://freesound.org/apiv2/oauth2/access_token/ \
     -d client_id=CLIENT_ID -d client_secret=CLIENT_SECRET \
     -d grant_type=authorization_code -d code=THE_CODE
   ```
4. Export the `access_token` it returns:
   ```bash
   export FREESOUND_ACCESS_TOKEN=xxxxxxxxxxxxxxxxxxxx
   ```

## Common tasks

### Rebuild the entire sound set from scratch
```bash
export FREESOUND_ACCESS_TOKEN=...
./download_and_convert.sh          # → sounds_organized/<category>/<id>.ogg
./repackage.sh 1.0.0               # → ambient-sounds-bundle.zip + manifest
```

### Add one or more new sounds
```bash
export FREESOUND_ACCESS_TOKEN=...
# ./add_sound.sh <freesound_id> <category> <id> "<Display Name>"
./add_sound.sh 210540 animals crickets_meadow "Crickets in a Meadow"
./add_sound.sh 274259 nature  waterfall_soft  "Soft Waterfall"
```
Each call verifies the freesound id, checks the license is CC0/CC-BY,
downloads + converts into the right category folder, and appends a catalog
entry.

### Publish a new version
```bash
./repackage.sh 1.1.0 --publish     # rebuild + create GitHub Release sounds-v1.1.0
git add VERIFIED_CATALOG.json BUNDLE_MANIFEST.json
git commit -m "feat: ambient sounds bundle v1.1.0"
git push
```
`--publish` uploads both the ZIP and the loose OGGs to the Release. Always
commit the updated manifest so the app knows what SHA to verify against.

## Why the audio isn't in git

- The full set is ~700MB; a single sound can exceed **GitHub's 100MB per-file
  commit limit**, and 700MB of binaries in git history is painful to clone.
- freesound downloads require **per-user OAuth**, so an end user can't fetch
  the originals — the app downloads our **processed** bundle from our own host.
  GitHub Releases (2GB/file) is that host.
- The scripts + catalog fully regenerate the audio, so the binaries are build
  output, not source.

## Runtime shape (Rust crate)

Follows the same download-engine shape as `rust-thesaurus` /
`rust-languagetool`, but **module-level, all-at-once** rather than per-item:
one bundle download, one `SoundState` (`NotInstalled`/`Downloading`/`Ready`/
`Error`), verified against `BUNDLE_MANIFEST.json`'s SHA-256, extracted into
`appDataDir/ambient-sounds/<category>/<id>.ogg`. Plus save/recall for
user-defined and preset sound mixes.

## Licensing

Every sound is **CC0** or **CC-BY 3.0** (author recorded per-entry in the
catalog). `add_sound.sh` refuses anything else. CC-BY sounds must keep their
author credit in the app's attribution surface.
