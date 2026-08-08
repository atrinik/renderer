#!/usr/bin/env bash
set -euo pipefail

test -n "${RUNNER_TEMP:-}"
test -n "${GITHUB_PATH:-}"

sudo apt-get update
sudo apt-get install --no-install-recommends --yes \
  gcc-mingw-w64-x86-64 libasound2-dev libdbus-1-dev libdecor-0-dev \
  libdrm-dev libegl1-mesa-dev libgbm-dev libgl1-mesa-dev libgles2-mesa-dev \
  libibus-1.0-dev libpipewire-0.3-dev libpulse-dev libsndio-dev libudev-dev \
  liburing-dev libvulkan-dev libwayland-dev libx11-dev libxcursor-dev \
  libxext-dev libxfixes-dev libxi-dev libxkbcommon-dev libxrandr-dev \
  libxss-dev libxtst-dev mesa-vulkan-drivers xvfb

install -d "${RUNNER_TEMP}/bin"
rustup toolchain install 1.97.1 --profile minimal --component clippy,rustfmt \
  --target x86_64-pc-windows-gnu

curl --fail --silent --show-error --location \
  https://github.com/anchore/syft/releases/download/v1.50.0/syft_1.50.0_linux_amd64.tar.gz \
  --output "${RUNNER_TEMP}/syft.tar.gz"
printf '%s  %s\n' bf7b29ff57f06da30918266a0e1c2885a8f99784798d1bdb1628886aa015d788 \
  "${RUNNER_TEMP}/syft.tar.gz" | sha256sum --check --strict
tar -xzf "${RUNNER_TEMP}/syft.tar.gz" -C "${RUNNER_TEMP}/bin" syft

curl --fail --silent --show-error --location \
  https://github.com/aquasecurity/trivy/releases/download/v0.73.0/trivy_0.73.0_Linux-64bit.tar.gz \
  --output "${RUNNER_TEMP}/trivy.tar.gz"
printf '%s  %s\n' 2edd39da482bb4e9831962487b68f68e3928ec3137794757f54d00383d79547b \
  "${RUNNER_TEMP}/trivy.tar.gz" | sha256sum --check --strict
tar -xzf "${RUNNER_TEMP}/trivy.tar.gz" -C "${RUNNER_TEMP}/bin" trivy

printf '%s\n' "${HOME}/.cargo/bin" "${RUNNER_TEMP}/bin" >>"${GITHUB_PATH}"
