#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Some constants
################################################################################
readonly OUTPUT_DIR="/tmp/nomad-openapi-generated"
REPOSITORY_ROOT="$(git rev-parse --show-toplevel)"
readonly REPOSITORY_ROOT

################################################################################
# Variables for generation
################################################################################
# Nomad OpenAPI isn't versioned, so.... next best thing
NOMAD_OPENAPI_SHA="37d950c8b53d12000e65d82d24d19a2bee83ec9f" # Dec 6, 2021
OPENAPI_GENERATOR_CLI_VERSION="v5.3.1"

################################################################################
# Get the OpenAPI template
################################################################################
cd /tmp
rm -rf /tmp/nomad-openapi
git clone git@github.com:hashicorp/nomad-openapi.git
cd /tmp/nomad-openapi
git checkout "${NOMAD_OPENAPI_SHA}"

################################################################################
# Generate hyper library code into `/tmp/nomad-openapi-generated`
################################################################################
CRATE_NAME="nomad-client-gen"
CRATE_VERSION="1.0.0" # You can't do any cute version names, just numeric

# Learn about other `--additional-properties` at:
# https://github.com/OpenAPITools/openapi-generator/blob/master/docs/generators/rust.md

rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}"
docker run \
    --user "${UID}:${GID}" \
    --rm \
    -v "$PWD:/local" \
    -v "${OUTPUT_DIR}:/output" \
    openapitools/openapi-generator-cli:${OPENAPI_GENERATOR_CLI_VERSION} generate \
    --input-spec /local/v1/openapi.yaml \
    --output /output/ \
    --generator-name rust \
    --library reqwest \
    --additional-properties=packageName="${CRATE_NAME}" \
    --additional-properties=packageVersion="${CRATE_VERSION}" \
    --additional-properties=useSingleRequestParameter=true

################################################################################
# Modify the generated code a bit
################################################################################
# Add a note about how this was generated.
echo "This folder was generated with 'make generate-nomad-rust-client'" \
    > "${OUTPUT_DIR}/GENERATED.md"

# Disable warnings since it's a generated library
readonly LIB_RS="${OUTPUT_DIR}/src/lib.rs"
echo -e "#![allow(warnings)]\n$(cat "${LIB_RS}")" > "${LIB_RS}"

# Use rustls, not native-tls
readonly CARGO_TOML="${OUTPUT_DIR}/Cargo.toml"

# We have to define FEATURES_OVERRIDE like this (with the explicit
# `\n`) in order to use it as a substitution in the subsequent `sed`
# command. Trying to use a multiline heredoc doesn't work.
readonly FEATURES_OVERRIDE='default_features = false\nfeatures = ["json", "multipart", "rustls-tls"]'
sed --in-place "s/features.*/${FEATURES_OVERRIDE}/g" "${CARGO_TOML}"

# We don't need any scripting to push anything to Github; that's
# already taken care of, thank you very much.
rm -f "${OUTPUT_DIR}/git_push.sh"

# We don't use Travis CI
rm -f "${OUTPUT_DIR}/.travis.yml"

# We've already got a .gitignore file for our Rust code, and this
# doesn't add anything.
rm -f "${OUTPUT_DIR}/.gitignore"

################################################################################
# Copy library into src/rust
################################################################################
rm -rf "${REPOSITORY_ROOT}/src/rust/${CRATE_NAME}"
mv "${OUTPUT_DIR}" "${REPOSITORY_ROOT}/src/rust/${CRATE_NAME}"
