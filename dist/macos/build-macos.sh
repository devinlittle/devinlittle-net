#!/bin/bash
set -euo pipefail

GIT_COUNT=$(git rev-list --count HEAD)
GIT_SHORT=$(git rev-parse --short=7 HEAD)
VERSION="r${GIT_COUNT}.${GIT_SHORT}"

echo "Building macOS package with version: ${VERSION}"

APP_NAME="dln"
BUNDLE_NAME="${APP_NAME}.app"
PKG_DIR="../target/packaging/macos"

rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}/${BUNDLE_NAME}/Contents/"{MacOS,Resources}

cp ../target/aarch64-apple-darwin/release/dln-cli "${PKG_DIR}/${BUNDLE_NAME}/Contents/MacOS/dlncli"
cp ../target/aarch64-apple-darwin/release/dln-ui "${PKG_DIR}/${BUNDLE_NAME}/Contents/MacOS/dln-ui"
chmod +x "${PKG_DIR}/${BUNDLE_NAME}/Contents/MacOS/"*

mkdir -p "${PKG_DIR}/${BUNDLE_NAME}/Contents"

sed "s/VERSION_HERE/${VERSION}/g" ../dist/macos/Info.plist >"${PKG_DIR}/${BUNDLE_NAME}/Contents/Info.plist"

cat >"${PKG_DIR}/${BUNDLE_NAME}/Contents/MacOS/dln" <<'EOF'
#!/bin/sh
HERE="$(dirname "$0")"

case "$1" in
    cli|--cli|-cli)
        shift
        exec "$HERE/dlncli" "$@"
        ;;
    ui|--ui|-ui)
        shift
        exec "$HERE/dln-ui" "$@"
        ;;
    -h|--help|help)
        echo "DLN Application Toolkit"
        echo "Usage:"
        echo "  DLN.app/Contents/MacOS/dln          -> Launch GUI"
        echo "  DLN.app/Contents/MacOS/dln cli      -> Launch CLI"
        echo "  DLN.app/Contents/MacOS/dln ui       -> Launch GUI explicitly"
        exit 0
        ;;
    *)
        exec "$HERE/dln-ui" "$@"
        ;;
esac
EOF

chmod +x "${PKG_DIR}/${BUNDLE_NAME}/Contents/MacOS/dln"

cd "${PKG_DIR}"
zip -r "../${APP_NAME}_${VERSION}_macos.zip" "${BUNDLE_NAME}"
echo "Created: target/packaging/${APP_NAME}_${VERSION}_macos.zip"
