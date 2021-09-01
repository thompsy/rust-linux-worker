FROM rust:latest as cargo-build
RUN apt-get update
RUN apt-get install musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup component add rustfmt
WORKDIR /usr/src/rust-linux-worker
COPY Cargo.toml Cargo.toml
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

# Note: this will fail but that's ok. We just want to cache this layer to avoid continually rebuilding dependencies
RUN cargo build || echo "ok"
#RUN rm -f target/x86_64-unknown-linux-musl/release/deps/rust-linux-worker*
COPY build.rs .
COPY ./src/ src/
RUN cargo build --bin server

FROM debian:latest
COPY --from=cargo-build /usr/src/rust-linux-worker/target/debug/server /server
EXPOSE 50051/tcp
CMD ["/server"]

