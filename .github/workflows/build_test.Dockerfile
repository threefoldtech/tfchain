FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y \
        build-essential \
        clang \
        cmake \
        curl \
        git \
        librocksdb-dev \
        libclang-dev \
        lld \
        lldb && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    $HOME/.cargo/bin/rustup install nightly-2022-05-11

RUN /bin/bash