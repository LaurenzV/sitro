#!/bin/bash
set -euo pipefail

IMAGE="vallaris/sitro-backends"
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
LOCK_FILE="$ROOT_DIR/docker/backend-image.lock"
cd "$ROOT_DIR"

PACKAGE_ID="$(cargo pkgid --package sitro)"
VERSION="${PACKAGE_ID##*#}"
VERSION="${VERSION##*@}"

if [[ "$VERSION" == *+* ]]; then
    echo "Cargo versions containing build metadata (+) cannot be used as Docker tags: $VERSION" >&2
    exit 1
fi

write_lock() {
    local digest="$1"

    if [[ ! "$digest" =~ ^sha256:[0-9a-f]{64}$ ]]; then
        echo "Invalid image digest: $digest" >&2
        exit 1
    fi

    local lock_file_tmp
    lock_file_tmp="$(mktemp "$LOCK_FILE.XXXXXX")"
    printf '%s:%s@%s\n' "$IMAGE" "$VERSION" "$digest" > "$lock_file_tmp"
    mv "$lock_file_tmp" "$LOCK_FILE"
    echo "Pinned $IMAGE:$VERSION to $digest in docker/backend-image.lock"
}

if [[ "${1:-}" == "--sync-lock" ]]; then
    if [[ "$#" -ne 1 ]]; then
        echo "Usage: $0 [--sync-lock]" >&2
        exit 1
    fi

    DIGEST="$(
        docker buildx imagetools inspect "$IMAGE:$VERSION" |
            awk '$1 == "Digest:" { print $2; exit }'
    )"
    write_lock "$DIGEST"
    exit 0
elif [[ "$#" -ne 0 ]]; then
    echo "Usage: $0 [--sync-lock]" >&2
    exit 1
fi

if docker buildx imagetools inspect "$IMAGE:$VERSION" >/dev/null 2>&1; then
    echo "$IMAGE:$VERSION already exists and must not be overwritten." >&2
    echo "Use '$0 --sync-lock' only if that image is the intended release." >&2
    exit 1
fi

TEST_IMAGE="$IMAGE:release-test-$VERSION"
docker buildx build \
    -t "$TEST_IMAGE" \
    -f docker/Dockerfile \
    --load \
    .
SITRO_DOCKER_IMAGE="$TEST_IMAGE" cargo test --workspace --all-features

METADATA_FILE="$(mktemp)"
trap 'rm -f "$METADATA_FILE"' EXIT

docker buildx build \
    --platform linux/amd64,linux/arm64 \
    -t "$IMAGE:$VERSION" \
    -f docker/Dockerfile \
    --metadata-file "$METADATA_FILE" \
    --push \
    .

DIGEST="$(
    sed -nE \
        's/^[[:space:]]*"containerimage.digest":[[:space:]]*"(sha256:[0-9a-f]{64})",?$/\1/p' \
        "$METADATA_FILE"
)"
write_lock "$DIGEST"
