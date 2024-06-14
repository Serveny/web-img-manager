################
##### Builder
FROM rust:1.79.0-slim as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new web-img-manager 

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/web-img-manager/

# Set the working directory
WORKDIR /usr/src/web-img-manager

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

# Install tools
RUN apt update && apt install -y musl-tools musl-dev
RUN apt-get install -y build-essential
RUN yes | apt install gcc-x86-64-linux-gnu

# set correct linker
ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release

# Now copy in the rest of the sources
COPY src /usr/src/web-img-manager/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/web-img-manager/src/main.rs

# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release

################
##### Runtime
FROM alpine:3.20 AS runtime 

# Copy application binary from builder image
COPY --from=builder /usr/src/web-img-manager/target/x86_64-unknown-linux-musl/release/web-img-manager /usr/local/bin

EXPOSE 8080 

# Run the application
CMD ["/usr/local/bin/web-img-manager"]
