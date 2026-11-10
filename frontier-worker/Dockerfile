FROM rust:1.99-slim AS build
WORKDIR /build
COPY . /snapshot
COPY frontier-worker/Cargo.toml frontier-worker/Cargo.toml
COPY frontier-worker/src frontier-worker/src
RUN cargo build --release --manifest-path frontier-worker/Cargo.toml
RUN cargo fetch --locked --manifest-path "/snapshot/Ares WebSite : App/backend/Cargo.toml"
RUN cargo fetch --locked --manifest-path /snapshot/core_lm/Cargo.toml
RUN cargo fetch --locked --manifest-path /snapshot/globalmind-runtime/Cargo.toml

FROM rust:1.99-slim
COPY --from=build /build/frontier-worker/target/release/frontier-worker /usr/local/bin/frontier-worker
COPY --from=build /usr/local/cargo/registry /usr/local/cargo/registry
WORKDIR /work
