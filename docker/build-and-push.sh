#!/bin/bash
set -e

IMAGE="vallaris/sitro-backends"

docker build -t "$IMAGE:latest" -f docker/Dockerfile .
docker push "$IMAGE:latest"
