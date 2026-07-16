#!/bin/sh
# fetch-or-build.sh — herdr [[build]] step for herdr-git-graph.
# Tries to download a prebuilt release binary; falls back to cargo build --release.
set -u

repo="jorge-huxley/herdr-git-graph"

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root="${GG_REPO_ROOT:-$script_dir/..}"
cargo_toml="${GG_CARGO_TOML:-$repo_root/Cargo.toml}"
out="${GG_OUT:-$repo_root/target/release/herdr-git-graph}"
base_url="${GG_BASE_URL:-https://github.com/$repo/releases/download}"

have() { command -v "$1" >/dev/null 2>&1; }

build_from_source() {
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
  if ! have cargo; then
    echo "herdr-git-graph needs Rust to build, but cargo was not found. Install from https://rustup.rs then re-run: herdr plugin install jorge-huxley/herdr-git-graph" >&2
    exit 1
  fi
  exec cargo build --release
}

fallback() {
  echo "herdr-git-graph: $1 — building from source instead." >&2
  [ -n "${tmpdir:-}" ] && rm -rf "$tmpdir"
  build_from_source
}

download() {
  if have curl; then
    curl -fsSL -o "$2" "$1"
  elif have wget; then
    wget -q -O "$2" "$1"
  else
    return 127
  fi
}

sha256_of() {
  if have sha256sum; then
    sha256sum "$1" | awk '{print $1}'
  elif have shasum; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    return 127
  fi
}

os=$(uname -s 2>/dev/null || echo unknown)
arch=$(uname -m 2>/dev/null || echo unknown)
triple=""
case "$os" in
  Darwin)
    case "$arch" in
      arm64|aarch64) triple="aarch64-apple-darwin" ;;
      x86_64|amd64) triple="x86_64-apple-darwin" ;;
    esac
    ;;
  Linux)
    case "$arch" in
      x86_64|amd64) triple="x86_64-unknown-linux-musl" ;;
    esac
    ;;
esac
[ -n "$triple" ] || fallback "no prebuilt binary for $os/$arch"

version=$(grep -E '^version *= *"' "$cargo_toml" 2>/dev/null | head -n 1 | sed -E 's/^version *= *"([^"]+)".*/\1/')
[ -n "$version" ] || fallback "could not read version from $cargo_toml"

asset="herdr-git-graph-$triple"
tmpdir=$(mktemp -d 2>/dev/null) || fallback "could not create a temp dir"
trap 'rm -rf "$tmpdir"' EXIT

bin_url="$base_url/v$version/$asset"
sums_url="$base_url/v$version/SHA256SUMS"
tmpbin="$tmpdir/$asset"
tmpsums="$tmpdir/SHA256SUMS"

download "$bin_url" "$tmpbin" || fallback "prebuilt binary not available for v$version ($asset)"
download "$sums_url" "$tmpsums" || fallback "checksums not available for v$version"

expected=$(grep -E "^[0-9a-f]{64} [ *]$asset\$" "$tmpsums" 2>/dev/null | awk '{print $1}' | head -n 1)
[ -n "$expected" ] || fallback "no checksum listed for $asset"

actual=$(sha256_of "$tmpbin") || fallback "no sha-256 tool available"
if [ "$actual" != "$expected" ]; then
  fallback "checksum mismatch for $asset"
fi

chmod +x "$tmpbin"
mkdir -p "$(dirname "$out")"
mv -f "$tmpbin" "$out" || fallback "could not install the verified binary to $out"
echo "herdr-git-graph: installed prebuilt v$version ($triple), verified SHA-256."
exit 0
