# Atrinik renderer

This independently releasable MIT Rust workspace is the shared GPU renderer
for Atrinik clients, editors, previewers, and deterministic offscreen tools. It
defines a renderer-neutral scene and API, explicit resource-provider boundary,
software reference implementation, private wgpu backend, thin SDL3 presentation
bridge, UI seam, conformance corpus, and `atrinik-render` proof CLI.

No public scene or renderer API exposes SDL3 or wgpu handles. Consumer-specific
state, filesystem/network discovery, game rules, protocol messages, event loops,
and editor transactions remain outside this repository.

## Validate

The pinned baseline is Rust 1.97.1, edition 2024. SDL 3.4.14 is compiled from
the Cargo-locked source rather than selected from the host. The aggregate required check
is `Renderer validation`.

```sh
cargo build --locked --workspace
cargo test --locked --workspace --all-targets
cargo run --locked --package atrinik-render -- --version
cargo run --locked --package atrinik-render -- probe
cargo run --locked --package atrinik-render -- corpus
SDL_VIDEO_DRIVER=x11 xvfb-run -a -s '-screen 0 1024x768x24' \
  cargo run --locked --package atrinik-render --features sdl3 -- window
tools/validate.sh
```

`tools/validate.sh` runs formatting, strict Clippy, unit/doc tests, shader
validation, dependency/architecture/provenance policy, deterministic corpus
comparison, Linux Vulkan offscreen proof, Windows/D3D12 cross-check, release
build, SBOM creation, and a release dry run. The SDL bridge is an explicit
`sdl3` feature so the default proof CLI remains portable to the native Windows
toolchain while Linux validation exercises a real window under Xvfb.

## Offscreen proof

Output is a small header followed by 64×64 RGBA8 bytes. The path must not
exist; persistence is same-directory, durable, and no-clobber.

```sh
cargo run --locked --package atrinik-render -- offscreen /tmp/atrinik-proof.rgba
test "$(wc -c </tmp/atrinik-proof.rgba)" -eq 16404
rm /tmp/atrinik-proof.rgba
```

See [architecture and limits](docs/ARCHITECTURE.md), [GPU lifecycle](docs/GPU-LIFECYCLE.md),
[corpus policy](corpus/README.md), and [provenance](PROVENANCE.md).
