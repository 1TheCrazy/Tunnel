#!/bin/sh
# Install the latest Tunnel node release for this machine.
#
# Run this from a directory containing node.toml, for example:
#   curl -fsSL https://raw.githubusercontent.com/1TheCrazy/Tunnel/main/install/node/install.sh | sudo sh

set -eu

REPOSITORY='1TheCrazy/Tunnel'
CONFIG_SOURCE="$PWD/node.toml"
DESTINATION="$PWD/node"
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
    x86_64|amd64)
        ARCHITECTURE='x86_64'
        ;;
    aarch64|arm64)
        ARCHITECTURE='aarch64'
        ;;
    armv7l|armv7)
        ARCHITECTURE='armv7'
        ;;
    *) fail "unsupported CPU architecture: $(uname -m). Supported: x86_64, aarch64, armv7." ;;
esac

install_dependencies() {
    if command -v apt-get >/dev/null 2>&1; then
        apt-get update >/dev/null
        DEBIAN_FRONTEND=noninteractive apt-get install -y wireguard iptables resolvconf >/dev/null
    elif command -v dnf >/dev/null 2>&1; then
        dnf install -y wireguard-tools iptables openresolv >/dev/null
    elif command -v yum >/dev/null 2>&1; then
        yum install -y wireguard-tools iptables openresolv >/dev/null
    elif command -v pacman >/dev/null 2>&1; then
        pacman -Sy --noconfirm wireguard-tools iptables openresolv >/dev/null
    elif command -v zypper >/dev/null 2>&1; then
        zypper --non-interactive install wireguard-tools iptables openresolv >/dev/null
    elif command -v apk >/dev/null 2>&1; then
        apk add --no-cache wireguard-tools iptables openresolv >/dev/null
    else
        fail 'no supported package manager found (apt, dnf, yum, pacman, zypper, or apk).'
    fi
}

asset_url="https://github.com/$REPOSITORY/releases/latest/download/tunnel_node_$ARCHITECTURE"

install_dependencies

temporary_binary="$DESTINATION.download.$$"
trap 'rm -f "$temporary_binary"' EXIT HUP INT TERM
curl --fail --silent --show-error --location --output "$temporary_binary" "$asset_url" \
    || fail 'failed to download the node binary.'
[ -s "$temporary_binary" ] || fail 'downloaded node binary is empty.'
chmod +x "$temporary_binary"
mv -f "$temporary_binary" "$DESTINATION"
trap - EXIT HUP INT TERM

# When the script is piped into sudo sh, SUDO_USER is the invoking user. Fall
# back to the current user for direct root execution.
TARGET_USER=${SUDO_USER:-$(id -un)}
TARGET_HOME=$(awk -F: -v user="$TARGET_USER" '$1 == user { print $6; exit }' /etc/passwd)
[ -n "$TARGET_HOME" ] || fail "could not determine the home directory for $TARGET_USER"

TARGET_CONFIG_DIR="$TARGET_HOME/$CONFIG_DIR_NAME"
install -d -m 0755 -o "$TARGET_USER" -g "$(id -gn "$TARGET_USER")" "$TARGET_CONFIG_DIR"
install -m 0644 -o "$TARGET_USER" -g "$(id -gn "$TARGET_USER")" \
    "$CONFIG_SOURCE" "$TARGET_CONFIG_DIR/node.toml"
rm -f "$CONFIG_SOURCE"

printf '%s\n' 'Tunnel was successfully installed'
