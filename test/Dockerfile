FROM rust:alpine as builder
WORKDIR /usr/src
RUN apk add --no-cache musl-dev

# The standard way to build
#COPY . .
#RUN cargo build --release

# A workaround to avoid redownloading and recompling everything when only the source code changes
# This is in leui of a cargo build --deps-only command
COPY ./Cargo.toml ./Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {println!(\"if you see this, the build failed\")}" > src/main.rs \
    && cargo build
RUN rm src/main.rs

# Copy the source code and build the release binary
COPY ./src ./src
RUN cargo build --release

FROM alpine:latest
WORKDIR /root
COPY --from=builder /usr/src/target/release/resh .
CMD tail -f /dev/null
