FROM ghcr.dockerproxy.com/litcc/base:latest


COPY ca-certificates.crt /etc/ssl/certs/
# Import from builder.
ENV USER=unprivileged
ENV UID=1000
RUN adduser \
            --disabled-password \
            --gecos "" \
            --home "/nonexistent" \
            --shell "/sbin/nologin" \
            --no-create-home \
            --uid "${UID}" \
            "${USER}"

WORKDIR /app

# Copy our build
COPY static_package /app/package

# Use an unprivileged user.
USER unprivileged:unprivileged

LABEL authors="litcc"

CMD ["/app/package"]


