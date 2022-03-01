#!/usr/bin/env bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT

(
    cd "${THIS_DIR}"
    docker buildx bake rootfs-build
)

########################################
# Generate the rootfs.
# NOTE: We use Fuse to mount inside Docker, because loop devices are
# not supported on our developers' ChromeOS Crostini environments.
# https://support.google.com/chromebook/thread/17786448?hl=en
########################################
docker run \
    --rm \
    --device /dev/fuse \
    --cap-add SYS_ADMIN \
    --volume "${REPOSITORY_ROOT}/dist:/dist" \
    rootfs-build:dev