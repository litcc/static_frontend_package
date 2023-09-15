FROM scratch

# Import from builder.
COPY passwd /etc/passwd
COPY group /etc/group

WORKDIR /app

# Copy our build
COPY target/x86_64-unknown-linux-musl/release/static_package_upx /app/package

# Use an unprivileged user.
USER unprivileged:unprivileged


LABEL authors="litcc"



CMD ["/app/package"]


