# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.245.2/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends cmake

# DFINITY Canister smart contract SDK
RUN DFX_VERSION=0.14.0 sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"

# rust dfinity utility
# this command takes lot of time and resources,
# after the first time the container is built, try not to edit steps before to keep it cached
# and add new steps after this one
RUN cargo install ic-cdk-optimizer

RUN cargo install cargo-audit

# install wasm32-unknown-unknown
# https://internetcomputer.org/docs/current/developer-docs/build/cdks/cdk-rs-dfinity/rust-quickstart#register-build-and-deploy-your-project
RUN rustup target add wasm32-unknown-unknown