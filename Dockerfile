FROM rust:1.32.0-slim as build
COPY ./config /usr/local/cargo/config

# base init
RUN dpkg --add-architecture armhf
RUN apt-get update
RUN apt-get install -qq gcc-arm-linux-gnueabihf
RUN apt-get install -qq libdbus-1-dev:armhf
RUN apt-get install -qq binutils-arm-linux-gnueabi
RUN rustup target add armv7-unknown-linux-gnueabihf

# ENV PKG_CONFIG_PATH $PKG_CONFIG_PATH:/usr/lib/arm-linux-gnueabihf/pkgconfig
# ENV PKG_CONFIG_ALLOW_CROSS 1
# end base init

# args setting
ARG project_name=iflyos_link
ARG workdir=/usr/src/app
# end args setting

# init
WORKDIR ${workdir}
# end init

# ADD ./lib/* /usr/arm-linux-gnueabihf/lib/
# ADD ./dbus/*.h /usr/arm-linux-gnueabihf/include/

# deps build
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN mkdir src/
RUN echo "fn main() {println!(\"base build\")}" > src/main.rs
RUN cargo build --release --target=armv7-unknown-linux-gnueabihf
RUN rm -rf target/release/deps/${project_name}*
# end deps build

COPY . .
CMD cargo build --release --target=armv7-unknown-linux-gnueabihf && /usr/bin/arm-linux-gnueabi-strip -s /usr/src/app/target/armv7-unknown-linux-gnueabihf/release/iflyos_link