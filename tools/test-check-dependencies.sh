#!/usr/bin/env bash
set -euo pipefail

repository=$(git rev-parse --show-toplevel)
cd "${repository}"

temporary=$(mktemp -d /tmp/atrinik-renderer-dependency-test.XXXXXX)
trap 'rm -rf -- "${temporary}"' EXIT

mkdir "${temporary}/bin"
cat >"${temporary}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
test "$*" = "metadata --locked --offline --all-features --format-version 1"
cat "${MOCK_CARGO_METADATA}"
EOF
chmod +x "${temporary}/bin/cargo"

run_check() {
  PATH="${temporary}/bin:${PATH}" \
    MOCK_CARGO_METADATA="$1" tools/check-dependencies.sh
}

updated="${repository}/tools/fixtures/dependency-metadata.json"
run_check "${updated}"

unapproved_license="${temporary}/unapproved-license.json"
jq '(.packages[] | select(.name == "transitive") | .license) =
  "MIT AND GPL-3.0-only"' \
  "${updated}" >"${unapproved_license}"
if run_check "${unapproved_license}"; then
  echo "dependency policy accepted an unapproved graph license" >&2
  exit 1
fi

unapproved_source="${temporary}/unapproved-source.json"
jq '(.packages[] | select(.name == "transitive") | .source) =
  "git+https://example.invalid/transitive?rev=0123456789abcdef#0123456789abcdef"' \
  "${updated}" >"${unapproved_source}"
if run_check "${unapproved_source}"; then
  echo "dependency policy accepted an unapproved graph source" >&2
  exit 1
fi

undeclared_direct="${temporary}/undeclared-direct.json"
jq '(.resolve.nodes[0].deps) +=
  [{"pkg":"registry+https://github.com/rust-lang/crates.io-index#transitive@2.0.0"}]' \
  "${updated}" >"${undeclared_direct}"
if run_check "${undeclared_direct}"; then
  echo "dependency policy accepted an undeclared direct dependency" >&2
  exit 1
fi
