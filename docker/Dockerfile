# This file is originally from https://github.com/rust-embedded/cross but modified to work with Ubuntu 20.10, gtk and libhandy.

FROM ubuntu:20.10

RUN apt-get update && apt-get install --assume-yes wget git

RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/cmake.sh
RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/common.sh
RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/lib.sh
RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/xargo.sh
RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/qemu.sh
RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/dropbear.sh
RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/linux-image.sh
RUN wget https://raw.githubusercontent.com/rust-embedded/cross/master/docker/linux-runner

RUN chmod +x ./cmake.sh
RUN chmod +x ./common.sh
RUN chmod +x ./lib.sh
RUN chmod +x ./xargo.sh
RUN chmod +x ./qemu.sh
RUN chmod +x ./dropbear.sh
RUN chmod +x ./linux-image.sh
RUN chmod +x ./linux-runner

COPY linux-image.sh.diff /
RUN git apply linux-image.sh.diff

RUN /common.sh

RUN /cmake.sh

RUN /xargo.sh

RUN apt-get install --assume-yes --no-install-recommends \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross

RUN /qemu.sh aarch64 softmmu

RUN /dropbear.sh

RUN /linux-image.sh aarch64

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="/linux-runner aarch64" \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    QEMU_LD_PREFIX=/usr/aarch64-linux-gnu \
    RUST_TEST_THREADS=1

ENV DEBIAN_FRONTEND=noninteractive

RUN dpkg --add-architecture arm64 && \
    apt-get update

RUN apt-get install --assume-yes libc6:arm64 -o APT::Immediate-Configure=0

RUN apt-get install --assume-yes libgtk-3-dev:arm64 x11proto-dev:arm64

ENV PKG_CONFIG_LIBDIR=/usr/local/lib/aarch64-linux-gnu/pkgconfig:/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig/ \
    PKG_CONFIG_ALLOW_CROSS=1

RUN apt-get install --assume-yes libhandy-1-0:arm64 libhandy-1-dev:arm64
