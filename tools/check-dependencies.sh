#!/usr/bin/env bash
set -euo pipefail

repository=$(git rev-parse --show-toplevel)
cd "${repository}"

jq -e '
  .schema_version == 1
  and (.direct_dependencies | length == 7)
  and all(.direct_dependencies[]; [.name,.version,.license,.purpose] | all(. != ""))
  and (.owner != "") and (.review_cadence != "")
  and (.eol_response != "") and (.validation != "")
' policy/dependencies.json >/dev/null

metadata=$(mktemp /tmp/atrinik-renderer-metadata.XXXXXX)
trap 'rm -f -- "${metadata}"' EXIT
cargo metadata --locked --offline --format-version 1 >"${metadata}"

jq -e --slurpfile policy policy/dependencies.json '
  . as $metadata
  | all($metadata.packages[];
      (.license // "") as $license
      | any($policy[0].allowed_spdx[];
          . as $allowed
          | $license
          | test("(^|[ (/])" + ($allowed | gsub("\\."; "\\.")) + "([ )/]|$)")))
    and all($policy[0].direct_dependencies[];
      . as $dependency
      | any($metadata.packages[];
          .name == $dependency.name
          and .version == $dependency.version
          and .license == $dependency.license))
' "${metadata}" >/dev/null
