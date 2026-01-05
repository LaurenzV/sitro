#!/bin/bash
set -e

IMAGE="vallaris/sitro-backends"

docker buildx build \
    --platform linux/amd64,linux/arm64 \
    -t "$IMAGE:latest" \
    -f docker/Dockerfile \
    --push \
    .
