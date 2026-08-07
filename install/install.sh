#!/bin/sh
# Usage: curl -fsSL <install-url> | sudo sh -s -- --node|--server|--cli

set -eu

REPOSITORY='1TheCrazy/Tunnel'
CONFIG_DIR_NAME='.config/1thecrazy/tunnel'

fail() {
    printf '%s\n' "error: $*" >&2
    exit 1
}

[ "$#" -eq 1 ] || fail 'usage: sh -- --node|--server|--cli'
INSTALL_KIND=$1

case "$INSTALL_KIND" in
    --node)
        ASSET_NAME='tunnel-node'
        BINARY_NAME='node'
        CONFIG_NAME='node.toml'
        ;;
    --server)
        ASSET_NAME='tunnel-server'
        BINARY_NAME='server'
        CONFIG_NAME='server.toml'
        ;;
    --cli)
        ASSET_NAME='tunnel-client-cli'
        BINARY_NAME='tunnel'
        CONFIG_NAME=''
        ;;
    *) fail 'usage: sh -- --node|--server|--cli' ;;
esac

command -v curl >/dev/null 2>&1 || fail 'curl is required to download the release.'
[ "$(id -u)" -eq 0 ] || fail 'run this installer as root (for example: curl ... | sudo sh -- --node).'

case "$(uname -s)" in
    Linux) ;;
    *) fail 'this installer supports Linux only.' ;;
esac

case "$(uname -m)" in
    x86_64|amd64) ARCHITECTURE='x86_64' ;;
    aarch64|arm64) ARCHITECTURE='aarch64' ;;
    armv7l|armv7) ARCHITECTURE='armv7' ;;
    armv6l|armv6) ARCHITECTURE='armv6' ;;
    *) fail "unsupported CPU architecture: $(uname -m). Supported: x86_64, aarch64, armv7, armv6." ;;
esac

install_dependencies() {
    if [ "$INSTALL_KIND" = '--server' ]; then
        return
    fi

    if command -v apt-get >/dev/null 2>&1; then
        apt-get update >/dev/null
        if [ "$INSTALL_KIND" = '--node' ]; then
            DEBIAN_FRONTEND=noninteractive apt-get install -y wireguard iptables resolvconf >/dev/null
        else
            DEBIAN_FRONTEND=noninteractive apt-get install -y wireguard >/dev/null
        fi
    elif command -v dnf >/dev/null 2>&1; then
        if [ "$INSTALL_KIND" = '--node' ]; then
            dnf install -y wireguard-tools iptables openresolv >/dev/null
        else
            dnf install -y wireguard-tools >/dev/null
        fi
    elif command -v yum >/dev/null 2>&1; then
        if [ "$INSTALL_KIND" = '--node' ]; then
            yum install -y wireguard-tools iptables openresolv >/dev/null
        else
            yum install -y wireguard-tools >/dev/null
        fi
    elif command -v pacman >/dev/null 2>&1; then
        if [ "$INSTALL_KIND" = '--node' ]; then
            pacman -Sy --noconfirm wireguard-tools iptables openresolv >/dev/null
        else
            pacman -Sy --noconfirm wireguard-tools >/dev/null
        fi
    elif command -v zypper >/dev/null 2>&1; then
        if [ "$INSTALL_KIND" = '--node' ]; then
            zypper --non-interactive install wireguard-tools iptables openresolv >/dev/null
        else
            zypper --non-interactive install wireguard-tools >/dev/null
        fi
    elif command -v apk >/dev/null 2>&1; then
        if [ "$INSTALL_KIND" = '--node' ]; then
            apk add --no-cache wireguard-tools iptables openresolv >/dev/null
        else
            apk add --no-cache wireguard-tools >/dev/null
        fi
    else
        fail 'no supported package manager found (apt, dnf, yum, pacman, zypper, or apk).'
    fi
}

if [ -n "$CONFIG_NAME" ]; then
    CONFIG_SOURCE="$PWD/$CONFIG_NAME"
    [ -f "$CONFIG_SOURCE" ] || fail "expected configuration file at $CONFIG_SOURCE"
fi

install_dependencies

asset_url="https://github.com/$REPOSITORY/releases/latest/download/${ASSET_NAME}-linux-$ARCHITECTURE"
if [ "$INSTALL_KIND" = '--cli' ]; then
    DESTINATION='/usr/local/bin/tunnel'
else
    DESTINATION="$PWD/$BINARY_NAME"
fi

temporary_binary=$(mktemp "${TMPDIR:-/tmp}/tunnel.download.XXXXXX")
trap 'rm -f "$temporary_binary"' EXIT HUP INT TERM
curl --fail --silent --show-error --location --output "$temporary_binary" "$asset_url" \
    || fail "failed to download the $BINARY_NAME binary."
[ -s "$temporary_binary" ] || fail "downloaded $BINARY_NAME binary is empty."
chmod +x "$temporary_binary"

if [ "$INSTALL_KIND" = '--cli' ]; then
    install -d -m 0755 /usr/local/bin
fi
mv -f "$temporary_binary" "$DESTINATION"
trap - EXIT HUP INT TERM

if [ -n "$CONFIG_NAME" ]; then
    TARGET_USER=${SUDO_USER:-$(id -un)}
    TARGET_HOME=$(awk -F: -v user="$TARGET_USER" '$1 == user { print $6; exit }' /etc/passwd)
    [ -n "$TARGET_HOME" ] || fail "could not determine the home directory for $TARGET_USER"

    TARGET_CONFIG_DIR="$TARGET_HOME/$CONFIG_DIR_NAME"
    TARGET_GROUP=$(id -gn "$TARGET_USER")
    install -d -m 0755 -o "$TARGET_USER" -g "$TARGET_GROUP" "$TARGET_CONFIG_DIR"
    install -m 0644 -o "$TARGET_USER" -g "$TARGET_GROUP" \
        "$CONFIG_SOURCE" "$TARGET_CONFIG_DIR/$CONFIG_NAME"
    rm -f "$CONFIG_SOURCE"
fi

printf '%s\n' 'Tunnel was successfully installed'
