# Atrinik renderer repository guide

## Mission and ownership

- This repository owns the fresh MIT-licensed shared Rust GPU renderer used by
  the connected client, native editor, deterministic offscreen tools, and
  world/region image generation.
- Keep renderer APIs source-neutral. They consume bounded renderer-owned scene,
  resource, target, camera, clock, and material inputs; they do not know whether
  a scene came from the Go server, a Rust client session, an editor document,
  or a fixture.
- `scene` and `render-api` stay independent of SDL3, wgpu, protocol, client,
  editor, and content syntax. The wgpu backend consumes those logical types
  while keeping devices, queues, surfaces, handles, pipelines, caches, and
  uploads private.
- The SDL3 bridge is a thin surface/raw-window integration boundary. It must not
  own an application event loop, input semantics, audio, client lifecycle, or
  editor workflow. Headless crates must not acquire an SDL3/GPU dependency
  unless their documented feature explicitly requires it.
- Never depend on `atrinik/client`, `atrinik/editor`, `atrinik/protocol`, or a
  content parser. Client and editor own their respective state/document-to-scene
  adapters; resource providers own filesystem/network policy outside this
  repository.
- Draw only the authorized records supplied by a consumer. Gameplay visibility,
  disclosure, collision, targeting authority, and world rules do not belong in
  rendering code.

## Rendering and API invariants

- Use one device/frame-graph/material/resource architecture for window,
  embedded viewport, and offscreen targets. Do not create consumer-specific or
  scenery-only renderers.
- Preserve deterministic ordered-scene behavior across neighboring tiles and
  physical depths, including tall/multipart objects, structural geometry,
  cutout/transparency, lighting, effects, overlays, clipping, and pixel-art
  sampling.
- Color output may use fixture-specific measured backend tolerances. Semantic
  identity, painter/depth, coverage, and visibility masks are exact contracts
  and are the canonical basis for picking, selection, disclosure, and robust
  cross-backend tests.
- Resource-provider APIs accept immutable IDs, digests, revisions, declared
  limits, and bounded asynchronous bytes. They never grant renderer code
  ambient filesystem or network access. Keep CPU, staging, and GPU cache
  ownership distinct with observable budgets and precise invalidation.
- Treat scene/resource/material/shader inputs as untrusted bounded data. Reject
  invalid revisions, dimensions, counts, coordinates, dependency graphs, and
  byte sizes before publishing partial frame state. A connected server must
  never be able to provide executable shaders or code.
- Surface and device loss, resize, suspend, target destruction, cancellation,
  and readback/output failures return typed lifecycle outcomes. Recovery creates
  one coherent new generation; stale GPU handles and partially successful
  outputs must not survive.
- Keep `atrinik-render`, consumer integrations, editor previews, imagery jobs,
  and golden tests on the same library paths. Offscreen tools take explicit,
  digest-verified inputs, perform no ambient discovery/network access, and
  publish outputs atomically.
- Isolate unavoidable wgpu/SDL raw-handle unsafe code in the smallest reviewed
  bridge, with documented safety invariants and lifecycle tests. Logical scene,
  API, resource-model, and testkit crates should forbid unsafe code.

## Roadmap and issue discipline

- The master replacement plan is `atrinik/atrinik#168`; repository issues and
  acceptance criteria are the executable source of truth. Link every change to
  an issue and M1-M6 milestone, and create a focused issue before introducing
  an unplanned rendering contract or backend.
- M1 establishes the Cargo workspace, dependency directions, clean-room visual
  corpus, and SDL3/wgpu proof for window, embedded, and offscreen output.
- M2 is the foundational delivery phase: device/target lifecycle, resource
  providers, ordered scenes, typography/UI integration, deterministic
  offscreen rendering, metrics, semantic masks, cross-depth snapshots, and the
  released shared SDL3/wgpu base.
- M3 has no separate renderer-owned feature lane. Support the first playable
  slice by stabilizing and releasing M2 contracts; do not hide untracked M3
  work in consumer pull requests.
- M4 builds scalable shared presentation: 2.5D geometry and occlusion,
  data-driven WGSL effects, retained/incremental GPU state, and projection/image
  products. Preserve the M2 scene and mask contracts or version changes
  explicitly.
- M5 adds bounded structural chunks merged with authoritative live overlays
  through the same retained scene.
- M6 owns coherent crate/CLI releases, image generation, fuzz/GPU soak,
  device-recovery, backend-equivalence, and production evidence.
- Work may proceed in parallel on scene types, device/targets, resources,
  passes, typography, fixtures, metrics, and consumer adapters only after
  issue #2 freezes dependency directions. Use fakes and reviewed draft types;
  integrate consumers through versioned releases, never copied source.

## Licensing, provenance, shaders, and assets

- New Rust code, WGSL shaders, tests, documentation, schemas, and newly authored
  fixtures in this repository are MIT. Do not add GPL/AGPL code dependencies or
  adapt legacy rendering source, tests, comments, or internal structure by
  default. Observable behavior and preserved product specifications may guide
  an independent implementation.
- Historical reuse is allowed only for a person and scope present in the
  exhaustive approved-grantor registry in the current `atrinik/atrinik`
  `AGENTS.md`. Apply its complete-history, identity, separability,
  third-party-review, and recording requirements exactly; fail closed on any
  incomplete history, mixed authorship, uncertain origin, or conflicting
  notice. Cite the exact wrapper revision containing the registry entry in the
  destination pull request or provenance manifest.
- Graphics, fonts, maps, content, and other fixture/resource inputs retain their
  exact licenses and attribution. Do not call a mixed corpus or generated image
  MIT merely because renderer code is MIT; generated products must carry the
  notices required by their source assets.
- Admit fixture and packaged assets only through a machine-readable manifest
  recording source, author, exact license, digest, transformation, and required
  notice. Review derivatives/composites against every input. Tests and releases
  must fail on ambiguous, incompatible, missing, or unacknowledged material.
- Shader packages are executable renderer code: author and review them here,
  version them with renderer compatibility, validate them in CI, and never load
  arbitrary server-delivered WGSL.

## Rust and GPU quality

- Pin stable Rust, edition, MSRV policy, `Cargo.lock`, wgpu/SDL3/native
  acquisition, supported Vulkan/Linux and D3D12/Windows backends, and explicit
  fallback policy. Record every dependency in the wrapper supply-chain
  inventory before relying on it.
- Once the Cargo workspace exists, every change must pass the aggregate
  `Renderer validation` contract: rustfmt, Clippy with warnings denied,
  workspace unit/integration/doc tests, architecture tests, shader validation,
  generated drift, dependency/license/security checks, consumer smoke builds,
  and applicable Linux/Windows build and rendering suites.
- Test logical scene/resource behavior headlessly. Use explicit render clocks,
  immutable fixture manifests, exact semantic masks, measured color tolerance,
  fake providers/targets, malformed/bounds cases, and lifecycle fault
  injection. Record adapter/backend/driver/limits for GPU evidence; never hide
  absent GPU coverage behind a passing software-only claim.
- Every performance-sensitive issue defines before/after fixtures and budgets
  for CPU/GPU time, uploads, allocations, cache/queue sizes, and recovery. Keep
  metric cardinality bounded and ensure disabled instrumentation is cheap.
- Treat warnings as errors; avoid ambient user state, source-tree writes,
  nondeterministic clocks, networked tests, and tests that require sibling
  consumer checkouts. Always run `git diff --check`.
- Use `atrinik/atrinik` profiles for coordinated consumer validation and local
  overrides when the wrapper registers this repository. Do not invent a direct
  source-copy workflow. Hand off exact client/editor/offscreen scenarios and
  state any display, GPU, backend, or software-adapter prerequisite.

## Packages, releases, and current repository state

- This repository independently owns a coherent version set for rendering
  crates, scene bundles, schemas, shader/material packages, fixtures, and the
  `atrinik-render` binary. Client and editor must be able to consume immutable
  releases without this source checkout.
- Releases include compatibility metadata, checksums, SBOM, provenance,
  licenses/notices, reproducibility evidence, supported Rust/platform/backend
  policy, and actionable driver/device diagnostics. Do not bundle assets unless
  explicitly allowlisted with their exact terms.
- Pull-request titles and squash commits use Conventional Commits. Every squash
  merge is released by semantic-release; do not create release tags manually
  or couple publication to a consumer or wrapper commit.
- The repository is currently a seed containing only licensing, roadmap
  documentation, and ignore policy. Until issue #1 lands Cargo and CI, do not
  claim that rustfmt, Clippy, tests, shader validation, SDL3/wgpu proofs, GPU
  backends, consumer builds, or release dry-runs ran. For seed-only
  documentation changes, inspect the complete tree, confirm the MIT boundary
  and links, and run `git diff --check`; report unavailable future checks
  honestly. After bootstrap, the repository-defined full validation is
  mandatory.
