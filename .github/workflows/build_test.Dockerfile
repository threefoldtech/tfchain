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
        zstd \
        wget \
        protobuf-compiler && \
    wget https://go.dev/dl/go1.20.2.linux-amd64.tar.gz && \
    tar -xvf go1.20.2.linux-amd64.tar.gz && \
    mv go /usr/local && \
    echo "GOPATH=/usr/local/go" >> ~/.bashrc && \
    echo "PATH=\$PATH:\$GOPATH/bin" >> ~/.bashrc && \
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