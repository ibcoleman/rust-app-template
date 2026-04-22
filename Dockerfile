FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin appuser

COPY app /usr/local/bin/app

ENV BIND_ADDR=0.0.0.0:8080 \
    RUST_LOG=info

EXPOSE 8080
USER appuser
ENTRYPOINT ["/usr/local/bin/app"]
