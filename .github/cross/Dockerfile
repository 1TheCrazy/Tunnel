ARG CROSS_BASE_IMAGE=ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main
FROM $CROSS_BASE_IMAGE

# Docker build arguments are scoped to the stage and must be redeclared after
# FROM. Cross passes this as arm64, armhf, or amd64 for the selected target.
ARG CROSS_DEB_ARCH

# Installing a foreign-architecture GUI package with dpkg would run its
# post-install scripts. Those scripts can execute a target binary (for example
# ARM64 Python) and fail on Cross's x64 builder. Download the resolved package
# set and unpack it into a sysroot instead: headers, libraries and pkg-config
# files are available to the linker, but no target binary is executed.
ENV PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_SYSROOT_DIR=/opt/cross-sysroot \
    PKG_CONFIG_PATH=/opt/cross-sysroot/usr/lib/x86_64-linux-gnu/pkgconfig:/opt/cross-sysroot/usr/lib/aarch64-linux-gnu/pkgconfig:/opt/cross-sysroot/usr/lib/arm-linux-gnueabihf/pkgconfig:/opt/cross-sysroot/usr/share/pkgconfig

RUN dpkg --add-architecture "$CROSS_DEB_ARCH" \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get --download-only install --yes --no-install-recommends \
        "libwebkit2gtk-4.1-dev:$CROSS_DEB_ARCH" \
        "libayatana-appindicator3-dev:$CROSS_DEB_ARCH" \
        "librsvg2-dev:$CROSS_DEB_ARCH" \
        "libssl-dev:$CROSS_DEB_ARCH" \
    && mkdir -p /opt/cross-sysroot \
    && for package in /var/cache/apt/archives/*.deb; do dpkg-deb --extract "$package" /opt/cross-sysroot; done \
    && cp -a /opt/cross-sysroot/lib/. /lib/ \
    && cp -a /opt/cross-sysroot/usr/lib/. /usr/lib/ \
    && rm -rf /var/lib/apt/lists/*
