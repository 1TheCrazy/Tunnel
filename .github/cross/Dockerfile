ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

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
