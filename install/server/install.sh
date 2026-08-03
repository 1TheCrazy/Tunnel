#!/bin/sh
# Install the latest Tunnel server release for this machine.
# Run this from a directory containing server.toml.

set -eu

REPOSITORY='1TheCrazy/Tunnel'
CONFIG_SOURCE="$PWD/server.toml"
DESTINATION="$PWD/server"
CONFIG_DIR_NAME='.config/1thecrazy/tunnel'

fail() {
    printf '%s\n' "error: $*" >&2
    exit 1
}

command -v curl >/dev/null 2>&1 || fail 'curl is required to download the release.'
[ "$#" -eq 0 ] || fail 'this installer does not accept arguments.'
[ "$(id -u)" -eq 0 ] || fail 'run this installer as root (for example: curl ... | sudo sh).'
[ -f "$CONFIG_SOURCE" ] || fail "expected configuration file at $CONFIG_SOURCE"

case "$(uname -s)" in
    Linux) ;;
    *) fail 'this installer supports Linux only.' ;;
esac

case "$(uname -m)" in
    x86_64|amd64) ARCHITECTURE='x86_64' ;;
    aarch64|arm64) ARCHITECTURE='aarch64' ;;
    armv7l|armv7) ARCHITECTURE='armv7' ;;
    *) fail "unsupported CPU architecture: $(uname -m). Supported: x86_64, aarch64, armv7." ;;
esac

asset_url="https://github.com/$REPOSITORY/releases/latest/download/tunnel_server_$ARCHITECTURE"
temporary_binary="$DESTINATION.download.$$"
trap 'rm -f "$temporary_binary"' EXIT HUP INT TERM
curl --fail --silent --show-error --location --output "$temporary_binary" "$asset_url" \
    || fail 'failed to download the server binary.'
[ -s "$temporary_binary" ] || fail 'downloaded server binary is empty.'
chmod +x "$temporary_binary"
mv -f "$temporary_binary" "$DESTINATION"
trap - EXIT HUP INT TERM

TARGET_USER=${SUDO_USER:-$(id -un)}
TARGET_HOME=$(awk -F: -v user="$TARGET_USER" '$1 == user { print $6; exit }' /etc/passwd)
[ -n "$TARGET_HOME" ] || fail "could not determine the home directory for $TARGET_USER"

TARGET_CONFIG_DIR="$TARGET_HOME/$CONFIG_DIR_NAME"
TARGET_GROUP=$(id -gn "$TARGET_USER")
install -d -m 0755 -o "$TARGET_USER" -g "$TARGET_GROUP" "$TARGET_CONFIG_DIR"
install -m 0644 -o "$TARGET_USER" -g "$TARGET_GROUP" \
    "$CONFIG_SOURCE" "$TARGET_CONFIG_DIR/server.toml"
rm -f "$CONFIG_SOURCE"

printf '%s\n' 'Tunnel was successfully installed'
