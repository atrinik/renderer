# Third-party notices

Rust dependencies retain their licenses as recorded in
`policy/dependencies.json`, Cargo metadata, and the release SBOM.

The raw SDL window handle adapter in `atrinik-render-sdl3` is an independently
adapted form of the public `sdl3` 0.18.4
`examples/raw-window-handle-with-wgpu/main.rs` technique. The upstream crate and
example are MIT licensed. The destination narrows it to a private synchronous
bridge, adds explicit safety invariants, and exposes no wgpu handle.

All scene fixtures, corpus descriptions, semantic masks, visual expectations,
and WGSL shader source are new synthetic MIT material authored for this
repository. No classic Atrinik implementation, media, authored game content,
or GPL/AGPL material is distributed here.
