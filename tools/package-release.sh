#!/usr/bin/env bash
set -euo pipefail

repository=$(git rev-parse --show-toplevel)
cd "${repository}"

output=${1:-dist}
if [[ -e ${output} ]]; then
  echo "release output already exists: ${output}" >&2
  exit 1
fi
install -d "${output}/crates"
target_directory=$(cargo metadata --locked --offline --no-deps --format-version 1 \
  | jq -r '.target_directory')

version=$(git describe --tags --always --dirty)
archive="atrinik-renderer-${version}.tar.gz"
git archive --format=tar --prefix="atrinik-renderer-${version}/" HEAD \
  | gzip -n >"${output}/${archive}"

cargo package --locked --offline --workspace --allow-dirty
cp "${target_directory}"/package/*.crate "${output}/crates/"
cargo build --locked --release --package atrinik-render
cp "${target_directory}/release/atrinik-render" "${output}/"
cp -R corpus "${output}/"
cp LICENSE PROVENANCE.md THIRD_PARTY_NOTICES.md "${output}/"

SYFT_CHECK_FOR_APP_UPDATE=false syft dir:. \
  --source-name atrinik-renderer --source-version "${version}" \
  --output "cyclonedx-json=${output}/sbom.cdx.json"

jq -n \
  --arg version "${version}" \
  --arg revision "$(git rev-parse HEAD)" \
  --arg rust "$(rustc --version)" \
  '{schema_version:1,version:$version,revision:$revision,
    tools:{rust:$rust},backends:["Vulkan","D3D12"],sdl_major:3}' \
  >"${output}/provenance.json"

checksums=$(mktemp /tmp/atrinik-renderer-checksums.XXXXXX)
trap 'rm -f -- "${checksums}"' EXIT
(
  cd "${output}"
  find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum
) >"${checksums}"
mv "${checksums}" "${output}/SHA256SUMS"

for crate in atrinik-render atrinik-render-api atrinik-render-resources \
  atrinik-render-sdl3 atrinik-render-testkit atrinik-render-ui \
  atrinik-render-wgpu atrinik-scene; do
  test -s "${output}/crates/${crate}-0.1.0.crate"
done
tar -tf "${output}/crates/atrinik-render-wgpu-0.1.0.crate" \
  | grep -Fx 'atrinik-render-wgpu-0.1.0/src/shader.wgsl' >/dev/null
