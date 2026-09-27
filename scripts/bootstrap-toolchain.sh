#!/usr/bin/env bash
# Install the island's build tools on Cloudflare Workers Builds.
#
# Sourced by build-island.sh, and only acts when WORKERS_CI is set: Cloudflare's
# build image ships Node and pnpm but no Rust, so every Workers Build of this
# repo died at `cargo: command not found` from the day it was connected
# (2026-09-11) — app.mzizi.dev kept serving a manual deploy while every build
# went red. On a contributor machine this does nothing; install the tools as
# CONTRIBUTING.md §1 describes instead of having a script do it behind you.
#
# Each tool is installed only if missing, into $HOME, from prebuilt binaries:
# `cargo install wasm-bindgen-cli` would add minutes of compile to every build.

[[ -n "${WORKERS_CI:-}" ]] || return 0

BINARYEN_VERSION=123
LOCAL_BIN="$HOME/.local/bin"
mkdir -p "$LOCAL_BIN"
export PATH="$HOME/.cargo/bin:$LOCAL_BIN:$PATH"

if ! command -v cargo >/dev/null 2>&1; then
  echo "bootstrap: installing Rust (stable, minimal, wasm32-unknown-unknown)"
  curl -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --profile minimal --target wasm32-unknown-unknown
elif ! rustup target list --installed 2>/dev/null | grep -qx wasm32-unknown-unknown; then
  rustup target add wasm32-unknown-unknown
fi

# The CLI must match the `wasm-bindgen` crate Cargo.lock resolved, exactly —
# wasm-bindgen refuses a module built against a different schema version. Read
# it from the lockfile so a dependency bump cannot leave this pin behind.
WASM_BINDGEN_VERSION=$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3; exit }' Cargo.lock)
if [[ -z "$WASM_BINDGEN_VERSION" ]]; then
  echo "bootstrap: could not read the wasm-bindgen version from Cargo.lock" >&2
  exit 1
fi
if [[ "$(wasm-bindgen --version 2>/dev/null)" != "wasm-bindgen $WASM_BINDGEN_VERSION" ]]; then
  echo "bootstrap: installing wasm-bindgen $WASM_BINDGEN_VERSION"
  name="wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl"
  curl -sSfL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/${name}.tar.gz" \
    | tar -xz -C "$LOCAL_BIN" --strip-components=1 "${name}/wasm-bindgen"
fi

if ! command -v wasm-opt >/dev/null 2>&1; then
  echo "bootstrap: installing wasm-opt (binaryen version_${BINARYEN_VERSION})"
  dir="$HOME/.local/binaryen"
  mkdir -p "$dir"
  curl -sSfL "https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz" \
    | tar -xz -C "$dir" --strip-components=1
  ln -sf "$dir/bin/wasm-opt" "$LOCAL_BIN/wasm-opt"
fi

echo "bootstrap: $(cargo --version) · $(wasm-bindgen --version) · $(wasm-opt --version)"
