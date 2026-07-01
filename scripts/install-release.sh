#!/usr/bin/env bash
set -euo pipefail

REPO="${OMNISTREAM_REPO:-}"
TAG="${OMNISTREAM_TAG:-latest}"
ARCH="${OMNISTREAM_ARCH:-linux-amd64}"
INSTALL_DIR="${OMNISTREAM_HOME:-$HOME/omnistream}"

usage() {
  cat <<'EOF'
Usage:
  install-release.sh --repo OWNER/REPO [--tag vX.Y.Z|latest] [--arch linux-amd64|linux-arm64] [--dir /opt/omnistream]

Examples:
  curl -fsSL https://raw.githubusercontent.com/OWNER/REPO/main/scripts/install-release.sh \
    | bash -s -- --repo OWNER/REPO --tag latest --arch linux-amd64 --dir /opt/omnistream

Environment overrides:
  OMNISTREAM_REPO, OMNISTREAM_TAG, OMNISTREAM_ARCH, OMNISTREAM_HOME
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO="$2"
      shift 2
      ;;
    --tag)
      TAG="$2"
      shift 2
      ;;
    --arch)
      ARCH="$2"
      shift 2
      ;;
    --dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$REPO" ]]; then
  echo "--repo OWNER/REPO is required"
  usage
  exit 1
fi

case "$ARCH" in
  linux-amd64|linux-arm64) ;;
  *)
    echo "unsupported arch: $ARCH"
    exit 1
    ;;
esac

for cmd in curl tar; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd"
    exit 1
  fi
done

ASSET_NAME="omnistream-${ARCH}.tar.gz"
if [[ "$TAG" == "latest" ]]; then
  DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
else
  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET_NAME}"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

echo "Downloading $ASSET_NAME ..."
if ! curl -fL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET_NAME"; then
  echo "failed to download release asset: $ASSET_NAME"
  echo "url: $DOWNLOAD_URL"
  echo "Please make sure the Release exists and contains this asset."
  exit 1
fi

mkdir -p "$INSTALL_DIR"
echo "Installing runtime files while preserving existing data under $INSTALL_DIR/data ..."
tar \
  --exclude='data' \
  --exclude='data/*' \
  --exclude='*/data' \
  --exclude='*/data/*' \
  -xzf "$TMP_DIR/$ASSET_NAME" \
  -C "$INSTALL_DIR" \
  --strip-components=1
chmod +x "$INSTALL_DIR/bin/server" "$INSTALL_DIR/scripts/release-start.sh" "$INSTALL_DIR/scripts/release-stop.sh"

echo "Installed OmniStream to $INSTALL_DIR"
echo "Preserved data directory: $INSTALL_DIR/data"
echo "Next steps:"
echo "  cd $INSTALL_DIR"
echo "  ./scripts/release-start.sh"
