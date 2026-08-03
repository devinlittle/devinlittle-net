mod apps
mod docs
mod services
mod website "frontend/website"

[private]
default: help

# shows this message:
help:
  @just --list

# updates dependencies
update:
  cargo update
  just update-lockfile
  bun update -r

# updates the flatpak cargo-sources.json
update-lockfile:
  cargo generate-lockfile
  uvx flatpak-cargo-generator ./Cargo.lock -o ./dist/linux/flatpak/cargo-sources.json

# build builer?

# runs a checkup
doctor proj="":
    #!/usr/bin/env bash
    set -e

    if [ -z "{{proj}}" ]; then
        echo "Checking all projects..."
        echo
        echo "[Apps]"
        just apps::doctor
        echo
        echo "[Services]"
        just services::doctor
        echo
        echo "[Website]"
        just website::doctor
        echo
        echo "[Docs]"
        just docs::doctor
        exit 0
    fi

    echo "Checking '{{proj}}'..."
    just {{proj}}::doctor
