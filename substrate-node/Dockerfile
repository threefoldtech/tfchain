# Note: We don't use Alpine and its packaged Rust/Cargo because they're too often out of date,
# preventing them from being used to build Substrate/Polkadot.

FROM phusion/baseimage:0.11 as builder
LABEL maintainer="chevdor@gmail.com"
LABEL description="This is the build stage for Substrate. Here we create the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
WORKDIR /substrate

COPY . /substrate

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake pkg-config libssl-dev git clang

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup install nightly-2020-10-01 && \
    rustup target add wasm32-unknown-unknown --toolchain nightly-2020-10-01 && \
	rustup default stable && \
	cargo +nightly-2020-10-01 build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer="chevdor@gmail.com"
LABEL description="This is the 2nd stage: a very small image where we copy the Substrate binary."
ARG PROFILE=release

RUN rm -rf /usr/share/* && \
	useradd -m -u 1000 -U -s /bin/sh -d /substrate substrate && \
	mkdir -p /substrate/.local && \
	chown -R substrate:substrate /substrate/.local

COPY --from=builder /substrate/target/$PROFILE/substrate /usr/local/bin

# checks
RUN ldd /usr/local/bin/substrate && \
	/usr/local/bin/substrate --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man && \
	rm -rf /src

USER substrate
EXPOSE 30333 9933 9944 9615
VOLUME ["/substrate"]

CMD ["/usr/local/bin/substrate", "--ws-external", "--rpc-external", "--rpc-cors=all", "--dev"]