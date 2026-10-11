# crates.io delivery

The public renderer family is `atrinik-scene`, `atrinik-render-resources`,
`atrinik-render-api`, `atrinik-render-ui`, `atrinik-render-testkit`,
`atrinik-render-wgpu`, and `atrinik-render-sdl3`. The policy allowlist in
`policy/crate-release.json` is authoritative. `atrinik-render` stays a GitHub
application artifact and has `publish = false`.

Cargo packages share the workspace version, initially `0.1.0`. This version is
independent of the semantic-release source tag. Change the workspace version and
all exact internal dependency requirements together whenever packaged content
changes after publication. Required validation compares packaged content with
the accepted base and rejects changed content at an unchanged published version.
A source-only release may reuse an already published package only when its
archive checksum matches exactly.

`tools/crate_prepare.py` runs without registry credentials. It prepares the
allowlisted Cargo archives, checks their normalized manifests and dependency
order, verifies standalone packages, and writes `crate-release.json` with the
source revision, source tag, package inventory and exact SHA-256 checksums.
The source release attaches each newly owned package archive, this manifest,
source provenance and checksums before dispatching the publisher. The manifest selects only new
crate versions; unchanged versions retain their original owning release. Preserve the CLI, corpus, SBOM and
source archive assets in that release.

`publish-crate.yml` runs from protected `main`. Its `automatic` dispatch accepts
the exact source release ID, full source revision and manifest digest. The
publisher checks release/tag/repository identity, verifies every asset and
normalized dependency record, exchanges GitHub OIDC in `crates-io-release`, and
uploads the stored archives directly to crates.io. It runs no Cargo build or
package hook. Source and application builds run first in a separate read-only
job without publication credentials; the source-release finalizer verifies the
prepared revision, source version and checksums before creating a tag. Dependencies must become visible with matching
checksums in the sparse index before dependent uploads proceed. Existing exact
packages are idempotent; a conflicting checksum stops publication. Hourly
reconciliation revisits eligible releases to recover interrupted dispatches or
index propagation delays.

## Immutable source-release ownership

Repository governance enables immutable GitHub releases before activation. The
publisher and owner bootstrap inspect the public source release and require
`immutable: true` before any registry write. The source finalizer verifies the
prepared assets without querying administrator-only repository settings. If
immutable releases were misconfigured, a mutable source release can exist, but
its crate publication fails closed. Preserve that release and its evidence for
explicit owner review; do not automatically delete, rewrite or replace assets.

## Initial ownership and activation

All seven names require an owner-run initial crates.io publication with a
narrowly scoped local API token, using the verified source-release archives.
Configure each crate's GitHub trusted publisher with this repository ID,
`publish-crate.yml` and environment `crates-io-release`. Keep the token out of
GitHub Actions, logs and committed files. Repository code prepares this rollout;
merging a candidate does not itself grant registry ownership or configure a
trusted publisher. The coordinator must verify initial versions and exact
checksums before activating automatic publication.

For a local preparation, run the pinned credential-free build image and use an
isolated Cargo cache:

```sh
python3 tools/crate_prepare.py check-version --base ACCEPTED_BASE_SHA
python3 tools/crate_prepare.py prepare --source-tag vSOURCE_VERSION --output /tmp/renderer-crates
```

For recovery, dispatch the existing eligible release through the same workflow
with its exact release ID, revision and manifest checksum. Never rebuild or
replace release archives to recover a failed upload. Correct conflicting
packages through a new reviewed Cargo version and source release.
