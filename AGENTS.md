# Atrinik renderer repository guide

## Ownership and architecture

- This repository owns the fresh MIT shared Rust GPU renderer for the connected
  client, native editor, deterministic offscreen tools, and world imagery.
- Renderer APIs consume bounded renderer-owned scenes, resources, targets,
  cameras, clocks, and materials; they remain source-neutral. `scene` and
  `render-api` must not depend on SDL3, wgpu, protocol, consumers, or content
  syntax. Backend GPU objects remain private.
- The SDL3 bridge owns only surface/raw-window integration, not application
  events, input, audio, client lifecycle, or editor workflows. Headless crates
  must not acquire native/GPU dependencies accidentally.
- Never depend on client/editor/protocol or content parsers. Consumers own
  state/document-to-scene adapters and authorization; rendering never decides
  gameplay visibility, collision, targeting, or disclosure.

## Rendering, resource, and lifecycle invariants

- Use one device/frame-graph/material/resource architecture for windows,
  embedded viewports, and offscreen targets. Do not create consumer-specific or
  scenery-only renderers.
- Preserve deterministic ordered scenes across neighboring tiles/depths,
  including tall/multipart objects, structural geometry, transparency,
  lighting, effects, overlays, clipping, and pixel-art sampling.
- Color snapshots may use measured backend tolerances. Semantic identity,
  painter/depth, coverage, and visibility masks are exact and own picking,
  selection, disclosure, and robust cross-backend tests.
- Resource providers accept immutable IDs/digests/revisions, declared limits,
  and bounded asynchronous bytes. They never grant ambient filesystem/network
  access. Keep CPU, staging, and GPU caches separately budgeted and precisely
  invalidated.
- Treat scene/resource/material/shader data as untrusted. Validate revisions,
  dimensions, counts, coordinates, dependency graphs, and byte sizes before
  publishing frame state. A server can never provide executable shaders/code.
- Return typed outcomes for device/surface loss, resize, suspend, cancellation,
  target destruction, readback, and output failure. Recovery creates one
  coherent generation; stale handles/partial outputs cannot survive.
- Keep CLI, consumer integration, previews, imagery, and golden tests on the
  same library paths. Offscreen tools use explicit digest-verified input, no
  ambient discovery/network, and atomic output.
- Isolate wgpu/SDL unsafe code to the smallest bridge with documented lifetime
  invariants and tests; logical crates should forbid unsafe code.

## Licensing, performance, and validation

- New Rust/WGSL/tests/docs/fixtures are MIT. Do not link, bundle, or depend on
  GPL/AGPL source as MIT. Historical Classic destination reuse follows the
  [local provenance record](PROVENANCE.md) and
  [canonical grant registry](https://github.com/atrinik/atrinik/blob/main/docs/PROVENANCE.md).
  At an exact source revision, prove every copyrightable portion is separable,
  within a recorded grant's past-contribution scope, and original work solely
  authored by its grantor. Present-day blame, majority authorship, a later
  edit, or an agent-assisted commit cannot fill a coverage gap. Admitted
  material may be inspected as implementation reference, copied, adapted,
  ported, translated, and MIT-relicensed in this destination. Record the exact
  source, destination, transformation, and registry revision; uncovered
  material fails closed. The Classic repository remains GPL-distributed.
- Authored inputs retain exact licenses. Fixture/package manifests record
  source, author, license, digest, transformation, and notice. Shaders are
  executable renderer code authored/reviewed/versioned here, never delivered
  arbitrarily by a server.
- Pin Rust/MSRV, `Cargo.lock`, wgpu/SDL acquisition, supported backends, and
  fallback policy. Every performance change defines representative before/
  after budgets for CPU/GPU time, uploads, allocations, caches, queues, and
  recovery. Bound metrics and keep disabled instrumentation cheap.
- `atrinik/atrinik#168` is the program roadmap; local issues/milestones own
  delivery. Do not copy the M1-M6 schedule here.
- Run the real aggregate contract:

  ```sh
  tools/validate.sh
  git diff --check
  ```

  `Renderer validation` owns formatting, strict Clippy, tests/docs,
  architecture/shader/generated drift, dependency/license/security gates,
  consumer smoke builds, and supported platform/rendering proofs. Record
  adapter/backend/driver/limits for GPU evidence; never imply absent GPU
  coverage passed.
- Wrapper replacement adapters are not available yet. Use repository
  validation and released consumer contracts, not unaudited source copies or
  Classic runtime fallbacks. Commits/PR titles use Conventional Commits;
  semantic-release owns coherent releases/tags.
