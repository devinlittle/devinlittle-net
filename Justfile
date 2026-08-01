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

# runs a checkup 
doctor proj="":
    #!/usr/bin/env bash
    set -e

    echo "Running environment checks..."

    if [ -z "{{proj}}" ]; then
        echo "Checking all projects..."
        just apps::doctor
        just services::doctor
        just website::doctor
        just docs::doctor
        exit 0
    fi

    echo "Checking '{{proj}}'..."
    just {{proj}}::doctor
