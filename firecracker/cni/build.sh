#!/usr/bin/env bash
set -euo pipefail

# ensure the cni directory exists that nomad-firecracker expects
sudo mkdir --parents /etc/cni/conf.d

sudo