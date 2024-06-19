################
##### Builder
################

FROM rust:latest as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new web-img-manager 

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/web-img-manager/

# Set the working directory
WORKDIR /usr/src/web-img-manager

# This is a dummy build to get the dependencies cached.
RUN cargo build --release

# Now copy in the rest of the sources
COPY src /usr/src/web-img-manager/src/
COPY config /usr/src/web-img-manager/config/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/web-img-manager/src/main.rs

# This is the actual application build.
RUN cargo build --release




################
##### Runtime
################

FROM ubuntu:latest AS runtime 

# Install nano
RUN apt update && apt install -y nano

VOLUME "/wim-storage"

# Copy application binary from builder image
COPY --from=builder /usr/src/web-img-manager/target/release/web-img-manager /usr/local/bin/

# Copy default config to volume
COPY --from=builder /usr/src/web-img-manager/config/default-server-config.json /wim-storage/config/default-server-config.json

# Create certificates folder
RUN USER=root mkdir /wim-storage/cert/

# Create default picture folder
RUN USER=root mkdir /wim-storage/pictures/

# link to config folder
RUN ln -s /wim-storage/config/ /usr/local/bin/

EXPOSE 1871 

# Set workdir to folder with binary and config link
WORKDIR /usr/local/bin/

# Run the application
CMD ["/usr/local/bin/web-img-manager"]
