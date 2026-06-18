# Building..

## Prerequesits:
  * Install the FreeDesktop SDK and rust Extension
  * Install flatpak-builder

  ```bash
flatpak install flathub org.freedesktop.Platform//25.08 org.freedesktop.Sdk//25.08 org.freedesktop.Sdk.Extension.rust-stable//25.08
```

## Building:

```bash
  flatpak-builder --force-clean --repo=repo build-dir net.devinlittle.dln.json
  ```

