#!/bin/bash
#
# > make a file executable
# chmod +x ./install.sh
set -e

DEFAULT_DIRNAME="$HOME/suno"
TEMPDIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMPDIR"
}
trap cleanup EXIT

read -p "> Enter the installation path, otherwise $DEFAULT_DIRNAME will be used: " DIRNAME
# Shorthand for "if empty, use default"
DIRNAME="${DIRNAME:-$DEFAULT_DIRNAME}"

echo "√ Output directory $DIRNAME"

# Determine the architecture and set the filename accordingly
ARCH=$(uname -m)
case $ARCH in
    x86_64) TARGET=x86_64-unknown-linux-gnu ;;
    arm64)  TARGET=aarch64-apple-darwin ;;
    *)         echo "ERROR: Unknown architecture $ARCH"; exit 1 ;;
esac

# Fetch the latest version from GitHub releases
LATEST_VERSION=$(curl -L -s -H 'Accept: application/json' https://github.com/turboflakes/suno/releases/latest | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')

if [ -z "$LATEST_VERSION" ] || [ "$LATEST_VERSION" == "null" ]; then
    echo "ERROR: Could not fetch latest version info"
    exit 1
fi

TARBALL_FILENAME=suno-$TARGET.tar.gz
TARBALL_FILENAME_SHA256=suno-$TARGET.tar.gz.sha256
URI="https://github.com/turboflakes/suno/releases/download/$LATEST_VERSION/$TARBALL_FILENAME"
URI_SHA256="https://github.com/turboflakes/suno/releases/download/$LATEST_VERSION/$TARBALL_FILENAME_SHA256"

echo "> Downloading suno $LATEST_VERSION"
cd "$TEMPDIR"

wget -q --show-progress "$URI" -O $TARBALL_FILENAME || { echo "ERROR: Failed to download the $TARBALL_FILENAME file"; exit 1; }
wget -q --show-progress "$URI_SHA256" -O $TARBALL_FILENAME_SHA256 || { echo "ERROR: Failed to download $TARBALL_FILENAME_SHA256 file"; exit 1; }

# Cross-platform checksum (Mac/Linux)
CHECK_CMD="sha256sum"
if ! command -v sha256sum &> /dev/null; then CHECK_CMD="shasum -a 256"; fi

if $CHECK_CMD -c "$TARBALL_FILENAME_SHA256" 2>&1 | grep -q 'OK'; then
    echo "√ Checksum verified"
    mkdir -p "$DIRNAME"

    # Backup existing binary
    FILENAME="$DIRNAME/suno"
    if [[ -f "$FILENAME" ]]; then
        mv "$FILENAME" "$FILENAME.backup"
        echo "√ Existing binary backed up to $FILENAME.backup"
    fi

    # Extract the tarball
    tar xzvf $TEMPDIR/$TARBALL_FILENAME suno
    echo "> Checking if suno exists: $(ls -l suno)"

    if [[ ! -f suno ]]; then
        echo "ERROR: Binary suno does not exist"; exit 1
    fi

    # Install suno at $DIRNAME´
    install -m 755 suno "$FILENAME"

    echo "√ Successfully installed suno $LATEST_VERSION at $FILENAME"

else
    echo "ERROR: SHA256 checksum verification failed";
fi
