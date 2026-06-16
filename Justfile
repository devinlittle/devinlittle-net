update:
  cargo update
  uvx flatpak-cargo-generator ./Cargo.lock -o ./dist/linux/flatpak/cargo-sources.json
  bun update
