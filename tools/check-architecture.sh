#!/usr/bin/env bash
set -euo pipefail

repository=$(git rev-parse --show-toplevel)
cd "${repository}"

metadata=$(mktemp /tmp/atrinik-renderer-architecture.XXXXXX)
actual=$(mktemp /tmp/atrinik-renderer-dependencies.XXXXXX)
trap 'rm -f -- "${metadata}" "${actual}"' EXIT
cargo metadata --locked --offline --all-features --format-version 1 >"${metadata}"

jq -S '
  . as $metadata
  | [$metadata.resolve.nodes[]
    | select(.id | startswith("path+file://"))
    | . as $node
    | ($metadata.packages[] | select(.id == $node.id) | .name) as $name
    | select($name | startswith("atrinik-"))
    | {key: $name,
       value: [$node.deps[].pkg as $id
         | $metadata.packages[] | select(.id == $id) | .name] | sort}]
  | from_entries
' "${metadata}" >"${actual}"

diff -u \
  <(jq -S '.crate_dependencies' policy/architecture.json) \
  "${actual}"

jq -e --slurpfile architecture policy/architecture.json '
  all(.packages[];
    (.source // "") as $source
    | all($architecture[0].forbidden_source_patterns[];
        . as $pattern | ($source | contains($pattern) | not)))
' "${metadata}" >/dev/null

if rg -n '\b(wgpu|sdl3)::' \
  crates/atrinik-scene crates/atrinik-render-api \
  crates/atrinik-render-resources crates/atrinik-render-ui; then
  echo "renderer-neutral crates expose a backend symbol" >&2
  exit 1
fi
