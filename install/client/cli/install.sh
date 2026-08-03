#!/bin/sh
# Install the Tunnel command-line client system-wide.

set -eu

REPOSITORY='1TheCrazy/Tunnel'
DESTINATION='/usr/local/bin/tunnel'

fail() {
    printf '%s\n' "error: $*" >&2
    exit 1
}

command -v curl >/dev/null 2>&1 || fail 'curl is required to download the release.'
[ "$#" -eq 0 ] || fail 'this installer does not accept arguments.'
[ "$(id -u)" -eq 0 ] || fail 'run this installer as root (for example: curl ... | sudo sh).'

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

install_wireguard() {
    if command -v apt-get >/dev/null 2>&1; then
        apt-get update >/dev/null
        DEBIAN_FRONTEND=noninteractive apt-get install -y wireguard >/dev/null
    elif command -v dnf >/dev/null 2>&1; then
        dnf install -y wireguard-tools >/dev/null
    elif command -v yum >/dev/null 2>&1; then
        yum install -y wireguard-tools >/dev/null
    elif command -v pacman >/dev/null 2>&1; then
        pacman -Sy --noconfirm wireguard-tools >/dev/null
    elif command -v zypper >/dev/null 2>&1; then
        zypper --non-interactive install wireguard-tools >/dev/null
    elif command -v apk >/dev/null 2>&1; then
        apk add --no-cache wireguard-tools >/dev/null
    else
        fail 'no supported package manager found (apt, dnf, yum, pacman, zypper, or apk).'
    fi
}

install_wireguard

asset_url="https://github.com/$REPOSITORY/releases/latest/download/tunel_client_cli_$ARCHITECTURE"
temporary_binary=$(mktemp "${TMPDIR:-/tmp}/tunnel.download.XXXXXX")
trap 'rm -f "$temporary_binary"' EXIT HUP INT TERM
curl --fail --silent --show-error --location --output "$temporary_binary" "$asset_url" \
    || fail 'failed to download the CLI binary.'
[ -s "$temporary_binary" ] || fail 'downloaded CLI binary is empty.'
chmod +x "$temporary_binary"
install -d -m 0755 /usr/local/bin
mv -f "$temporary_binary" "$DESTINATION"
trap - EXIT HUP INT TERM

printf '%s\n' 'Tunnel was successfully installed'
