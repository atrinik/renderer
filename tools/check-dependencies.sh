#!/usr/bin/env bash
set -euo pipefail

repository=$(git rev-parse --show-toplevel)
cd "${repository}"

jq -e '
  def nonempty_string: type == "string" and length > 0;

  .schema_version == 2
  and (.allowed_spdx
    | type == "array" and length > 0
      and all(.[]; nonempty_string) and length == (unique | length))
  and (.allowed_registry_sources
    | type == "array" and length > 0
      and all(.[]; nonempty_string) and length == (unique | length))
  and (.direct_dependencies | type == "array" and length > 0)
  and all(.direct_dependencies[];
    type == "object" and (.name | nonempty_string))
  and all(.direct_dependencies[];
    type == "object" and (.purpose | nonempty_string))
  and (.direct_dependencies | map(.name) | length == (unique | length))
  and (.owner | nonempty_string) and (.review_cadence | nonempty_string)
  and (.eol_response | nonempty_string) and (.validation | nonempty_string)
' policy/dependencies.json >/dev/null

metadata=$(mktemp /tmp/atrinik-renderer-metadata.XXXXXX)
trap 'rm -f -- "${metadata}"' EXIT
cargo metadata --locked --offline --all-features --format-version 1 >"${metadata}"

jq -e --slurpfile policy policy/dependencies.json '
  def license_identifiers:
    [scan("[A-Za-z0-9][A-Za-z0-9.-]*")
      | select(. != "AND" and . != "OR" and . != "WITH")];

  . as $metadata
  | ($metadata.workspace_members | unique) as $workspace_members
  | [
      $metadata.resolve.nodes[]
      | select(.id as $id | $workspace_members | index($id))
      | .deps[].pkg
    ] | unique as $direct_ids
  | [
      $metadata.packages[]
      | select(.id as $id | $direct_ids | index($id))
      | select(.id as $id | $workspace_members | index($id) | not)
      | .name
    ] | unique | sort as $direct_names
  | ($policy[0].direct_dependencies | map(.name) | unique | sort) as $policy_names
  | $direct_names == $policy_names
    and all($metadata.packages[];
      (.license // "" | license_identifiers) as $licenses
      | ($licenses | length > 0)
      and all($licenses[]; . as $license | $policy[0].allowed_spdx | index($license)))
    and all($metadata.packages[];
      if (.id as $id | $workspace_members | index($id))
      then .source == null
      else .source as $source | $policy[0].allowed_registry_sources | index($source)
      end)
' "${metadata}" >/dev/null
