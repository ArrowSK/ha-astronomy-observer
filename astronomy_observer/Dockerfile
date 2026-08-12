FROM rust:1.89-alpine3.22 AS rust-builder
WORKDIR /build
RUN apk add --no-cache musl-dev
COPY Cargo.toml ./
COPY src ./src
COPY web ./web
COPY dashboard ./dashboard
RUN cargo build --release

FROM alpine:3.22 AS astronomy-builder
ARG ASTRONOMY_ENGINE_COMMIT=61dc07020aaa6885d2c7f688a4d82beaf6edb9ef
RUN apk add --no-cache build-base curl
WORKDIR /build
RUN curl --fail --silent --show-error --location \
      "https://raw.githubusercontent.com/cosinekitty/astronomy/${ASTRONOMY_ENGINE_COMMIT}/source/c/astronomy.c" -o astronomy.c \
 && curl --fail --silent --show-error --location \
      "https://raw.githubusercontent.com/cosinekitty/astronomy/${ASTRONOMY_ENGINE_COMMIT}/source/c/astronomy.h" -o astronomy.h
COPY scripts/astro_helper.c ./astro_helper.c
RUN cc -Os -ffunction-sections -fdata-sections -Wl,--gc-sections -s \
      -o astro-helper astro_helper.c astronomy.c -lm

FROM alpine:3.22 AS catalogue-builder
ARG OPENNGC_COMMIT=da90466031b0372c896588b85be6016c617e205b
RUN apk add --no-cache python3 curl
WORKDIR /build
RUN curl --fail --silent --show-error --location \
      "https://raw.githubusercontent.com/mattiaverga/OpenNGC/${OPENNGC_COMMIT}/database_files/NGC.csv" -o NGC.csv \
 && curl --fail --silent --show-error --location \
      "https://raw.githubusercontent.com/mattiaverga/OpenNGC/${OPENNGC_COMMIT}/database_files/addendum.csv" -o addendum.csv
COPY scripts/build_catalog.py ./build_catalog.py
RUN python3 build_catalog.py NGC.csv addendum.csv catalog.tsv

FROM ghcr.io/home-assistant/base:3.24
ARG BUILD_VERSION=0.1.0
ARG BUILD_ARCH
LABEL io.hass.version="${BUILD_VERSION}" \
      io.hass.type="app" \
      io.hass.arch="${BUILD_ARCH}" \
      org.opencontainers.image.title="Astronomy Observer" \
      org.opencontainers.image.description="Observing conditions and target recommendations for amateur astronomy" \
      org.opencontainers.image.source="https://github.com/ArrowSK/ha-astronomy-observer" \
      org.opencontainers.image.licenses="PolyForm-Noncommercial-1.0.0"

RUN apk add --no-cache ca-certificates libgcc
COPY --from=rust-builder /build/target/release/astronomy-observer /usr/local/bin/astronomy-observer
COPY --from=astronomy-builder /build/astro-helper /usr/local/bin/astro-helper
COPY --from=catalogue-builder /build/catalog.tsv /usr/share/astronomy-observer/catalog.tsv
COPY data/meteor_showers.csv /usr/share/astronomy-observer/meteor_showers.csv
COPY run.sh /run.sh
RUN chmod 0755 /run.sh /usr/local/bin/astronomy-observer /usr/local/bin/astro-helper \
 && /usr/local/bin/astronomy-observer --self-test
CMD ["/run.sh"]
