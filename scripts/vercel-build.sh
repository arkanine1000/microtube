#!/usr/bin/env bash
#
# Production build for the MicroTube web app on Vercel.
#
# Vercel's build image ships Node but no Rust toolchain, so this script
# installs rustup + the wasm32 target + wasm-pack on demand, then runs the
# normal workspace build (Wasm core -> worklet bundle -> Vite). It is safe
# to run locally too — each install step is skipped when already present.
set -euo pipefail

CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"

# --- Rust toolchain --------------------------------------------------------
if ! command -v cargo >/dev/null 2>&1; then
  echo "--- installing Rust (rustup, minimal profile) ---"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
fi
export PATH="$CARGO_BIN:$PATH"

if command -v rustup >/dev/null 2>&1; then
  echo "--- adding wasm32-unknown-unknown target ---"
  rustup target add wasm32-unknown-unknown
else
  # No rustup (e.g. a distro-packaged Rust): assume the wasm32 std is
  # already provided by the system toolchain.
  echo "--- rustup absent; assuming wasm32 target is system-provided ---"
fi

# --- wasm-pack -------------------------------------------------------------
if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "--- installing wasm-pack (prebuilt) ---"
  curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh || true
  export PATH="$CARGO_BIN:$PATH"
  if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "--- prebuilt unavailable; building wasm-pack from source ---"
    cargo install wasm-pack --locked
  fi
fi

echo "--- toolchain ready: $(rustc --version), $(wasm-pack --version) ---"

# --- build -----------------------------------------------------------------
echo "--- building workspace ---"
npm run build
