FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive
COPY clean_disk_space.sh clean_disk_space.sh
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
        lldb \
        python3 \
        python3-pip \
        tar \
        zstd && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    $HOME/.cargo/bin/rustup install nightly-2022-05-11 && \
    # cleanup image 
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean && \
    apt-get autoclean && \
    apt-get autoremove && \
    rm -rf /tmp/* 
RUN /bin/bash