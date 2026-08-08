# Provenance review

This repository is clean-room MIT work. No classic Atrinik renderer, client,
editor, media, or content source was consulted or reused. Consequently no
historical Atrinik provenance grant is applied.

The only adapted implementation technique is the SDL3 raw-window-handle bridge
identified in `THIRD_PARTY_NOTICES.md`. Its exact source is the MIT-licensed
`sdl3` crate version 0.18.4 from crates.io, file
`examples/raw-window-handle-with-wgpu/main.rs`. Dependency identity and checksum
are fixed by `Cargo.lock`; the dependency is also retained normally rather than
vendored. Review found no embedded third-party asset or incompatible license.

The corpus is generated solely from constants in
`atrinik-render-testkit::synthetic_scene`. Its immutable identity string is
`atrinik-renderer-synthetic-resource-v1`. The fixture contains colored
rectangles only and derives from no prior Atrinik visual. `corpus/manifest.json`
records exact RGBA and semantic-plane digests, clock, dimensions, and the sole
allowed cross-backend visual tolerance.
