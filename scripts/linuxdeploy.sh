#!/usr/bin/env bash

# INFO: this is assuming the script is being ran in anywere 1 level above the root (i.e when cwd is == "/apps")
cd "../dist/linux/appimage/"

LINUX_DEPLOY_URL=""
LINUX_DEPLOY_QT_URL=""

case "$(uname -s)" in
Linux*)
  if [[ "$(uname -m)" == "aarch64" ]]; then
    LINUX_DEPLOY_URL=https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-aarch64.AppImage
    LINUX_DEPLOY_QT_URL=https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/1-alpha-20250213-1/linuxdeploy-plugin-qt-aarch64.AppImage
  else
    LINUX_DEPLOY_URL=https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage
    LINUX_DEPLOY_QT_URL=https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/1-alpha-20250213-1/linuxdeploy-plugin-qt-x86_64.AppImage
  fi
  ;;
*)
  echo "Must be on linux to run this script :("
  exit 0
  ;;
esac

echo "Downloading LinuxDeploy" &&
  curl \
    -LOJ \
    --progress-bar \
    --remote-header-name \
    "$LINUX_DEPLOY_URL"

echo "Downloading LinuxDeploy QT" &&
  curl \
    -LOJ \
    --progress-bar \
    --remote-header-name \
    "$LINUX_DEPLOY_QT_URL"

chmod +x *.AppImage

# CREATING APP DIR SECTION
#cd "$(pwd)/../dist/linux/appimage"
APPDIR=$(echo $(pwd)/AppDir)
ROOT="$(echo $(pwd)/../../..)"

rm -rf "$APPDIR"

mkdir -p \
  "$APPDIR/usr/bin" \
  "$APPDIR/usr/share/applications" \
  "$APPDIR/usr/share/metainfo" \
  "$APPDIR/usr/share/icons/hicolor"

# LINUX DESKTOP STUFF
install -Dm755 $ROOT/dist/linux/appimage/AppRun \
  "$APPDIR/usr/bin/AppRun"

install -Dm644 $ROOT/dist/linux/dln.desktop \
  "$APPDIR/usr/share/applications/net.devinlittle.dln.desktop"

install -Dm644 $ROOT/dist/linux/net.devinlittle.dln.metainfo.xml \
  "$APPDIR/usr/share/metainfo/net.devinlittle.dln.metainfo.xml"

install -Dm644 $ROOT/dist/icons/512x512/net.devinlittle.dln.png \
  "$APPDIR/usr/share/icons/hicolor/512x512/apps/net.devinlittle.dln.png"

install -Dm644 $ROOT/dist/icons/256x256/net.devinlittle.dln.png \
  "$APPDIR/usr/share/icons/hicolor/256x256/apps/net.devinlittle.dln.png"

install -Dm644 $ROOT/dist/icons/128x128/net.devinlittle.dln.png \
  "$APPDIR/usr/share/icons/hicolor/128x128/apps/net.devinlittle.dln.png"

install -Dm644 $ROOT/dist/icons/64x64/net.devinlittle.dln.png \
  "$APPDIR/usr/share/icons/hicolor/64x64/apps/net.devinlittle.dln.png"

install -Dm755 $ROOT/target/aarch64-unknown-linux-gnu/release/dln-cli \
  "$APPDIR/usr/bin/dlncli"

install -Dm755 $ROOT/target/aarch64-unknown-linux-gnu/release/dln-ui \
  "$APPDIR/usr/bin/dln-ui"

export QMAKE="$(pwd)/../../../scripts/qmake"
export QML_SOURCES_PATHS="$(pwd)/../../../apps/dln-ui/qml"
export QT_QPA_PLATFORM="wayland;xcb"
export DEPLOY_QT_PLUGIN=1

QT_LIBS=$($ROOT/scripts/qmake -query QT_INSTALL_LIBS)
export LD_LIBRARY_PATH="$QT_LIBS:/usr/lib/:/usr/lib/aarch64-linux-gnu/"

QMAKE="$(pwd)/../../../scripts/qmake" ./linuxdeploy-$(uname -m).AppImage \
  --appdir AppDir \
  --custom-apprun AppRun \
  --plugin qt \
  --output appimage
