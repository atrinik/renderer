# Provenance review

The current repository foundation is independently authored MIT work. No
Classic Atrinik renderer, client, editor, media, or content source was consulted
or reused for this revision. Consequently no historical Atrinik provenance
grant is applied to it.

Independent implementation remains the default where exact reuse is not
proven. Before Classic source is inspected as implementation reference or
reused, the
[canonical grant registry](https://github.com/atrinik/atrinik/blob/main/docs/PROVENANCE.md)
requires an exact source revision and a complete, non-shallow, rename-aware
audit. Every copyrightable portion of the selected separable material must fall
within a grant's recorded past-contribution scope and be original work solely
authored by that grantor. Present-day blame, majority authorship, a later edit,
or an agent-assisted commit cannot establish missing historical coverage; the
audit must also exclude copied, generated, vendored, embedded third-party, and
conflicting-licensed origins.

Only admitted material may be copied, adapted, ported, translated, and
MIT-relicensed in this destination. Record the exact source, destination,
transformation, third-party review, grantor, and canonical registry revision.
Every uncovered portion is excluded. The destination grant neither changes the
Classic repository's GPL distribution nor authorizes using its GPL source as a
renderer build/runtime dependency.

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
