FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && \
    apt install -y \
        build-essential \
        clang \
        cmake \
        curl \
        git \
        librocksdb-dev \
        libclang-dev \
        lld \
        lldb \
        software-properties-common \
        tar \
        zstd && \
    add-apt-repository ppa:deadsnakes/ppa && \
    apt install -y python3.10 && \
    curl https://bootstrap.pypa.io/get-pip.py > get-pip.py && \
    python3.10 get-pip.py && \
    rm -rf get-pip.py && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    $HOME/.cargo/bin/rustup install nightly-2022-05-11 && \
    # cleanup image 
    rm -rf /var/lib/apt/lists/* && \
    apt -y clean && \
    apt -y autoclean && \
    apt -y autoremove && \
    rm -rf /tmp/* 
RUN /bin/bash