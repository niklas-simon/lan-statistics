FROM --platform=$BUILDPLATFORM rust:1-alpine AS build

ARG BUILDPLATFORM
ARG TARGETARCH

RUN apk add --no-cache build-base musl-dev git zig \
 && cargo install --locked cargo-zigbuild

WORKDIR /app
COPY ./src-server/Cargo.toml ./src-server/Cargo.lock ./src-server/
COPY ./common/Cargo.toml ./common/Cargo.lock ./common/
RUN cd src-server \
 && mkdir -p src \
 && mkdir -p ../common/src \
 && echo 'fn main(){println!("stub");}' > src/main.rs \
 && touch ../common/src/lib.rs \
 && cargo fetch

COPY ./src-server ./common ./

RUN case "$TARGETARCH" in \
      "amd64")  export RUST_TARGET=x86_64-unknown-linux-musl ;; \
      "arm64")  export RUST_TARGET=aarch64-unknown-linux-musl ;; \
      *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac \
 && cd src-server \
 && rustup target add "$RUST_TARGET" \
 && cargo zigbuild --release --target "$RUST_TARGET" \
 && cp "target/${RUST_TARGET}/release/lan-manager" ./

FROM alpine
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=build /app/src-server/lan-manager ./

ENTRYPOINT ["./lan-manager"]