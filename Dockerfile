# https://dev.to/rogertorres/first-steps-with-docker-rust-30oi
FROM rust:1.59 as build

# create a new empty shell project
RUN USER=root cargo new --bin markhor
WORKDIR /markhor

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/markhor*
RUN cargo build --release

FROM debian:buster-slim

# copy the build artifact from the build stage
COPY --from=build /markhor/target/release/markhor .

WORKDIR /site
# set the startup command to run your binary
CMD ["/markhor"]
