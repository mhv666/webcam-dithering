FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

#RUN rustup target add armv7-unknown-linux-gnueabihf

RUN apt update && apt upgrade -y

RUN apt install -y llvm clang libopencv-dev clang libclang-dev cmake g++ wget unzip

COPY --from=planner /app/recipe.json recipe.json

#ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++

RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

RUN cargo build --release --bin webcam-dithering

# FROM rust

# COPY --from=builder /app/target/webcam-dithering /app/webcam-dithering

# RUN rustup target add armv7-unknown-linux-gnueabihf
# RUN rustup toolchain install stable-armv7-unknown-linux-gnueabihf

# WORKDIR /app

# ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_Linux_gnueabihf=arm-linux-gnueabihf-gcc CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++

# CMD ["cargo", "build", "--target", "armv7-unknown-linux-gnueabihf"]