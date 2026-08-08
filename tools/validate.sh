#!/usr/bin/env bash
set -euo pipefail

repository=$(git rev-parse --show-toplevel)
cd "${repository}"

test "$(rustc --version | cut -d' ' -f2)" = 1.97.1
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features
cargo test --locked --workspace --doc
cargo build --locked --release --package atrinik-render --features sdl3
cargo run --locked --quiet --package atrinik-render -- --version

tools/check-architecture.sh
tools/check-dependencies.sh
tools/check-provenance.sh
jq empty corpus/*.json policy/*.json

corpus_actual=$(mktemp /tmp/atrinik-renderer-corpus.XXXXXX)
proof_output=$(mktemp /tmp/atrinik-renderer-proof.XXXXXX)
proof_error=$(mktemp /tmp/atrinik-renderer-proof-error.XXXXXX)
rm -f -- "${proof_output}"
trap 'rm -f -- "${corpus_actual}" "${proof_output}" "${proof_error}"' EXIT
cargo run --locked --quiet --package atrinik-render-testkit --example corpus \
  >"${corpus_actual}"
diff -u \
  <(jq -S '{schema_version,cases}' corpus/manifest.json) \
  <(jq -S . "${corpus_actual}")

cargo run --locked --quiet --package atrinik-render -- probe
cargo run --locked --quiet --package atrinik-render -- corpus
SDL_VIDEO_DRIVER=x11 xvfb-run -a -s '-screen 0 1024x768x24' \
  cargo run --locked --quiet --package atrinik-render --features sdl3 -- window
cargo run --locked --quiet --package atrinik-render -- offscreen "${proof_output}"
test "$(wc -c <"${proof_output}")" -eq 16404
if cargo run --locked --quiet --package atrinik-render -- offscreen "${proof_output}" \
  2>"${proof_error}"; then
  echo "offscreen proof overwrote an existing output" >&2
  exit 1
fi
grep -F "without overwriting" "${proof_error}" >/dev/null

cargo check --locked --workspace --target x86_64-pc-windows-gnu \
  --exclude atrinik-render-sdl3

release_output=$(mktemp -d /tmp/atrinik-renderer-release.XXXXXX)
rmdir "${release_output}"
tools/package-release.sh "${release_output}"
test -s "${release_output}/SHA256SUMS"
rm -rf -- "${release_output}"

git diff --check
