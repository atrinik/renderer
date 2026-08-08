# M1 architecture and limits

Dependencies point inward toward contracts:

```text
scene <- resources <- render-api <- testkit
  ^          ^            ^          ^
  |          |            |          |
  +----------+------------+------ render-wgpu <- atrinik-render
  +-----------------------+------ render-sdl3
  +------------------------------ render-ui
```

`atrinik-scene` owns immutable, versioned, sorted render inputs. `render-api`
owns targets, lifecycle errors, output planes, and a renderer trait. Resource
providers receive explicit content identity, revision, SHA-256 digest, and byte
budget; this grants no ambient path or URL access. `render-testkit` supplies the
fake provider and exact CPU reference. wgpu device/queue/pipeline/texture types
remain private to `render-wgpu`; SDL owns presentation only. Consumers translate
their state into these contracts through adapters in their own repositories.

Default limits are 65,536 sprites, 8,192 pixels on either axis, 16,777,216 total
pixels, 8× scale, and 16 MiB per resource request. All float inputs must be
finite; color channels are within 0–1; zero identities/revisions and all-zero
digests are rejected. Semantic IDs are unique. Sorting is stable by layer,
depth, painter order, then semantic ID. Rectangles use half-open pixel coverage.

CPU work is O(pixels + sprites), GPU vertex generation is O(sprites), and GPU
readback is O(pixels). A completed frame owns 11 bytes per pixel across RGBA,
semantic, depth, and coverage planes. The GPU path additionally owns four bytes
per pixel for the color target, at most 255 bytes of row padding per row, and
144 bytes of transient vertices per sprite. Checked arithmetic precedes every
size-derived allocation.

No public dependency or type points to a client, editor, protocol, server, or
classic implementation. `tools/check-architecture.sh` validates exact direct
crate edges from Cargo metadata and rejects forbidden sibling sources.
