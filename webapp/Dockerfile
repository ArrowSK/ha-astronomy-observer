FROM rust:1.89-alpine3.22 AS rust-builder
WORKDIR /build/webapp
RUN apk add --no-cache musl-dev
COPY astronomy_observer/Cargo.toml astronomy_observer/Cargo.lock ./
COPY webapp/src ./src
COPY webapp/ui ./ui
COPY astronomy_observer/src /build/astronomy_observer/src
COPY astronomy_observer/web /build/astronomy_observer/web
COPY astronomy_observer/config.yaml /build/astronomy_observer/config.yaml
RUN cargo build --locked --release

FROM alpine:3.22 AS astronomy-builder
RUN apk add --no-cache build-base curl
WORKDIR /build
COPY astronomy_observer/scripts/astro_helper.c ./astro_helper.c
COPY webapp/build-astro-helper.sh ./build-astro-helper.sh
RUN chmod 0755 build-astro-helper.sh && ./build-astro-helper.sh

FROM alpine:3.22 AS catalogue-builder
RUN apk add --no-cache python3 curl
WORKDIR /build
COPY astronomy_observer/scripts/build_catalog.py ./build_catalog.py
COPY webapp/build-catalog.sh ./build-catalog.sh
RUN chmod 0755 build-catalog.sh && ./build-catalog.sh

FROM alpine:3.22
LABEL org.opencontainers.image.title="Astronomy Observer Web" \
      org.opencontainers.image.description="Standalone web deployment of Astronomy Observer" \
      org.opencontainers.image.source="https://github.com/ArrowSK/ha-astronomy-observer" \
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"
RUN apk add --no-cache ca-certificates libgcc \
 && mkdir -p /config /tmp/astronomy-observer-web /usr/share/astronomy-observer
COPY --from=rust-builder /build/webapp/target/release/astronomy-observer /usr/local/bin/astronomy-observer-web
COPY --from=astronomy-builder /build/astro-helper /usr/local/bin/astro-helper
COPY --from=catalogue-builder /build/catalog.tsv /usr/share/astronomy-observer/catalog.tsv
COPY astronomy_observer/data/meteor_showers.csv /usr/share/astronomy-observer/meteor_showers.csv
COPY astronomy_observer/data/world_atlas_3min.bin /usr/share/astronomy-observer/world_atlas_3min.bin
COPY astronomy_observer/data/world_atlas_3min.json /usr/share/astronomy-observer/world_atlas_3min.json
COPY astronomy_observer/data/WORLD_ATLAS_NOTICE.md /usr/share/astronomy-observer/WORLD_ATLAS_NOTICE.md
RUN chmod 0755 /usr/local/bin/astronomy-observer-web /usr/local/bin/astro-helper \
 && /usr/local/bin/astronomy-observer-web --self-test
EXPOSE 8080
CMD ["/usr/local/bin/astronomy-observer-web"]
