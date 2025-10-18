FROM rust as build

ARG TARGETPLATFORM
ARG TARGETARCH

RUN if [ "${TARGETARCH}" = "arm64" ]; then \
        rustup target add aarch64-unknown-linux-musl; \
    else \
        rustup target add x86_64-unknown-linux-musl; \
    fi && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates

COPY ./src-server ./src-server
COPY ./common ./common

RUN cd src-server && \
    if [ "${TARGETARCH}" = "arm64" ]; then \
        cargo build --target aarch64-unknown-linux-musl --release; \
        cp target/aarch64-unknown-linux-musl/release/lan-manager lan-manager; \
    else \
        cargo build --target x86_64-unknown-linux-musl --release; \
        cp target/x86_64-unknown-linux-musl/release/lan-manager lan-manager; \
    fi

FROM alpine
WORKDIR /app
COPY --from=build ./src-server/lan-manager lan-manager

ENTRYPOINT ["./lan-manager"]