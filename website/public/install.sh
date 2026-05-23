#!/bin/sh
set -eu

repo="${ASSURA_REPO:-rothnic/assura}"
version="${ASSURA_VERSION:-latest}"
bin_dir="${BIN_DIR:-$HOME/.local/bin}"

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "assura installer: missing required command: $1" >&2
    exit 1
  fi
}

need curl
need tar
need mktemp

os="$(uname -s)"
arch="$(uname -m)"

case "$os:$arch" in
  Linux:x86_64 | Linux:amd64)
    asset="assura-linux-amd64.tar.gz"
    ;;
  Darwin:arm64 | Darwin:aarch64)
    asset="assura-macos-arm64.tar.gz"
    ;;
  Darwin:x86_64 | Darwin:amd64)
    asset="assura-macos-amd64.tar.gz"
    ;;
  *)
    echo "assura installer: unsupported platform: $os/$arch" >&2
    echo "Download a matching archive from https://github.com/$repo/releases" >&2
    exit 1
    ;;
esac

if [ "${ASSURA_ASSET_URL:-}" ]; then
  url="$ASSURA_ASSET_URL"
elif [ "$version" = "latest" ]; then
  url="https://github.com/$repo/releases/latest/download/$asset"
else
  url="https://github.com/$repo/releases/download/$version/$asset"
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

echo "Downloading $url"
curl -fsSL "$url" -o "$tmp_dir/$asset"
tar -xzf "$tmp_dir/$asset" -C "$tmp_dir"

mkdir -p "$bin_dir"
install -m 755 "$tmp_dir/assura" "$bin_dir/assura"
install -m 755 "$tmp_dir/assura-full" "$bin_dir/assura-full"

echo "Installed assura to $bin_dir/assura"
case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *)
    echo "Add $bin_dir to PATH before running assura."
    ;;
esac
