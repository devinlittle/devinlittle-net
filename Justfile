update:
  cargo update
  just update-lockfile
  bun update

update-lockfile:
  cargo generate-lockfile
  uvx flatpak-cargo-generator ./Cargo.lock -o ./dist/linux/flatpak/cargo-sources.json

