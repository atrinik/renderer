#!/usr/bin/env bash
set -euo pipefail

repository=$(git rev-parse --show-toplevel)
cd "${repository}"

test -s PROVENANCE.md
test -s THIRD_PARTY_NOTICES.md
jq -e '
  .schema_version == 1
  and (.provenance | contains("synthetic MIT"))
  and (.cases | length == 2)
  and all(.cases[];
    .width > 0 and .height > 0 and .clock_millis == 1000
    and .maximum_rgba_channel_difference == 1
    and all(.digests[]; test("^[0-9a-f]{64}$")))
' corpus/manifest.json >/dev/null

if rg -ni '(classic[/ -](client|editor|renderer)|\b(a?gpl)-[123])' \
  crates corpus --glob '!*.lock'; then
  echo "unexpected classic or incompatible-license reference in implementation" >&2
  exit 1
fi
