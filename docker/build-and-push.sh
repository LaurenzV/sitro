#!/bin/bash
set -euo pipefail

IMAGE="vallaris/sitro-backends"
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

PACKAGE_ID="$(cargo pkgid --package sitro)"
VERSION="${PACKAGE_ID##*#}"
VERSION="${VERSION##*@}"

if [[ "$VERSION" == *+* ]]; then
    echo "Cargo versions containing build metadata (+) cannot be used as Docker tags: $VERSION" >&2
    exit 1
fi

docker buildx build \
    --platform linux/amd64,linux/arm64 \
    -t "$IMAGE:$VERSION" \
    -t "$IMAGE:latest" \
    -f docker/Dockerfile \
    --push \
    .
