# corylus-focus-noise

Ambient background-noise mixer engine for [Corylus](https://github.com/LudoBermejoES/corylus).

Vendored as a git submodule at `src-tauri/vendor/corylus-focus-noise` in the
main Corylus repository. Downloads a catalog of CC0/CC-BY ambient sound loops
(rain, waves, campfire, wind, etc.) on demand, verified by checksum, and
exposes save/recall for user-defined and preset sound mixes.

Follows the same download-engine shape as `rust-thesaurus` and
`rust-languagetool`: a static catalog, a per-item install state machine, and
a `provision(on_progress)` entry point the host app polls/subscribes to.
