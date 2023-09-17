FROM scratch

# Import from builder.
COPY passwd /etc/passwd
COPY group /etc/group

WORKDIR /app

# Copy our build
COPY static_package /app/package

# Use an unprivileged user.
USER unprivileged:unprivileged


LABEL authors="litcc"



CMD ["/app/package"]


