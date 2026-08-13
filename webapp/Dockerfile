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

FROM alpine:3.22
RUN apk add --no-cache ca-certificates libgcc
COPY --from=rust-builder /build/webapp/target/release/astronomy-observer /usr/local/bin/astronomy-observer-web
RUN chmod 0755 /usr/local/bin/astronomy-observer-web
EXPOSE 8080
CMD ["/usr/local/bin/astronomy-observer-web"]
