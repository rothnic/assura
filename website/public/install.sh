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
    if [ "${ASSURA_LINUX_LIBC:-}" = "musl" ] || \
      { command -v ldd >/dev/null 2>&1 && ldd --version 2>&1 | grep -qi musl; }; then
      asset="assura-linux-musl-amd64.tar.gz"
    else
      asset="assura-linux-amd64.tar.gz"
    fi
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

checksum_url="${ASSURA_CHECKSUM_URL:-$url.sha256}"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

download() {
  source_url="$1"
  destination="$2"

  case "$source_url" in
    file://*)
      cp "${source_url#file://}" "$destination"
      ;;
    *)
      if [ -f "$source_url" ]; then
        cp "$source_url" "$destination"
      else
        curl -fsSL "$source_url" -o "$destination"
      fi
      ;;
  esac
}

archive="$tmp_dir/$asset"
checksum="$tmp_dir/$asset.sha256"

echo "Downloading $url"
download "$url" "$archive"
echo "Verifying $asset"
download "$checksum_url" "$checksum"

expected="$(awk 'NF { print $1; exit }' "$checksum")"
case "$expected" in
  *[!0123456789abcdefABCDEF]* | "")
    echo "assura installer: invalid SHA-256 checksum for $asset" >&2
    exit 1
    ;;
esac
if [ "${#expected}" -ne 64 ]; then
  echo "assura installer: invalid SHA-256 checksum length for $asset" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$archive" | awk '{ print $1 }')"
elif command -v shasum >/dev/null 2>&1; then
  actual="$(shasum -a 256 "$archive" | awk '{ print $1 }')"
else
  echo "assura installer: missing SHA-256 command: sha256sum or shasum" >&2
  exit 1
fi

if [ "$expected" != "$actual" ]; then
  echo "assura installer: checksum mismatch for $asset" >&2
  exit 1
fi

tar -xzf "$archive" -C "$tmp_dir"

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
