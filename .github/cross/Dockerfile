ARG CROSS_BASE_IMAGE=ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main
FROM $CROSS_BASE_IMAGE

# Docker build arguments are scoped to the stage and must be redeclared after
# FROM. Cross passes this as arm64, armhf, or amd64 for the selected target.
ARG CROSS_DEB_ARCH

# CROSS_DEB_ARCH is supplied by Cross for the configured Rust target.
# These are build-time libraries; generated packages retain their normal
# runtime dependencies.
RUN dpkg --add-architecture "$CROSS_DEB_ARCH" \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install --yes --no-install-recommends \
        "libwebkit2gtk-4.1-dev:$CROSS_DEB_ARCH" \
        "libayatana-appindicator3-dev:$CROSS_DEB_ARCH" \
        "librsvg2-dev:$CROSS_DEB_ARCH" \
        "libssl-dev:$CROSS_DEB_ARCH" \
    && rm -rf /var/lib/apt/lists/*
