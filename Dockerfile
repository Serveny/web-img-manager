# Rust as the base image
FROM rust:1.78 as build

# Create a new empty shell project
# RUN USER=root cargo new --bin web-img-manager 
WORKDIR /web-img-manager

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy the admin-control-panel manifest
# COPY ./admin-control-panel/Cargo.toml ./admin-control-panel/Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY src ./src
#COPY ./admin-control-panel/src ./admin-control-panel/src

# Build for release.
RUN rm ./target/release/deps/web-img-manager*
# RUN cargo fetch --path .
RUN cargo build --release

# The final base image
FROM debian:bookworm

# Expose ports
EXPOSE 8080

# Copy from the previous build
COPY --from=build /web-img-manager/target/release/web-img-manager /usr/src/web-img-manager
# COPY --from=build /web-img-manager/target/release/web-img-manager/target/x86_64-unknown-linux-musl/release/web-img-manager.

# Run the binary
CMD ["/usr/src/web-img-manager"]