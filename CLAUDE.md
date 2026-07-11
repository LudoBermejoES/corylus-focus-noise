# corylus-focus-noise — Claude instructions

Ambient sound catalog + build pipeline for Corylus's Ambient Sounds module.
Read `README.md` first for the full workflow; this file is the quick operational
contract.

## Golden rules

1. **Never commit audio.** `sounds_organized/` and `ambient-sounds-bundle.zip`
   are gitignored on purpose (700MB, GitHub's 100MB/file limit, regenerable).
   Only text is committed: `VERIFIED_CATALOG.json`, `BUNDLE_MANIFEST.json`, the
   `*.sh` scripts, `src/`, docs. If you catch yourself `git add`-ing a `.ogg`
   or the `.zip`, stop.

2. **Audio ships via GitHub Releases, not the repo.** The ZIP (all sounds) and
   the loose per-sound OGGs (`<category>__<id>.ogg`) are Release assets. The app
   downloads the ZIP at runtime and verifies it against `BUNDLE_MANIFEST.json`.

3. **freesound is a build-time source only.** Its downloads need per-user
   OAuth2, so end users can't fetch originals. `FREESOUND_ACCESS_TOKEN` (24h)
   is required by the scripts — see README § Getting a token. Never hardcode a
   token or commit one.

4. **Every sound must be verified before inclusion.** `add_sound.sh` checks the
   freesound id resolves, prints its real name/author, and refuses non-CC0/CC-BY
   licenses. Do not add catalog entries by hand without this check — an earlier
   catalog was full of fabricated ids pointing at wrong/unrelated sounds. Trust
   the API, not guessed ids.

5. **Categories are fixed and each needs ≥6 sounds:** `animals`, `nature`,
   `noise`, `places`, `rain`, `things`, `transport`, `urban`. `repackage.sh`
   flags any category under 6.

## After changing the sound set

Always, in this order:
```bash
./repackage.sh <version>            # rebuild ZIP + manifest, cross-checks disk vs catalog
./repackage.sh <version> --publish  # when ready to ship: uploads ZIP + loose OGGs
git add VERIFIED_CATALOG.json BUNDLE_MANIFEST.json   # + any script/src/doc changes
```
The manifest SHA must match the published ZIP. `repackage.sh` recomputes both
from the same build, so re-run it (not a manual edit) whenever the audio
changes, and re-upload with `--clobber` if the ZIP was rebuilt.

## Consistency invariant

`VERIFIED_CATALOG.json` (what should exist) and `sounds_organized/` (what's on
disk) must agree. `repackage.sh` errors on a catalog entry with no file and
warns on a file with no catalog entry. Keep them in sync — the app trusts the
catalog.

## Versioning

Release tags are `sounds-vX.Y.Z`. Bump when the sound set changes. The Rust
crate's own version (Cargo.toml) is separate and follows the parent Corylus
repo's bump rules when the submodule is updated there.
