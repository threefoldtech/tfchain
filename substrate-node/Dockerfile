# Note: We don't use Alpine and its packaged Rust/Cargo because they're too often out of date,
# preventing them from being used to build Substrate/Polkadot.

FROM phusion/baseimage:0.11 as builder
LABEL maintainer="dylan@threefold.tech"
LABEL description="This is the build stage for Substrate. Here we create the binary."

ENV DEBIAN_FRONTEND=noninteractive

ARG PROFILE=release
WORKDIR /tfchain

COPY . /tfchain

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake pkg-config libssl-dev git clang

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup install nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	rustup default stable && \
	cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer="dylan@threefold.tech"
LABEL description="This is the 2nd stage: a very small image where we copy the tfchain binary."
ARG PROFILE=release

RUN rm -rf /usr/share/* && \
	mkdir -p /tfchain/.local

COPY --from=builder /tfchain/target/$PROFILE/tfchain /usr/local/bin
COPY --from=builder /tfchain/customSpecRaw.json /etc

# checks	
RUN ldd /usr/local/bin/tfchain && \
	/usr/local/bin/tfchain --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man && \
	rm -rf /src

EXPOSE 30333 9933 9944 9615
VOLUME ["/tfchain"]

ENTRYPOINT ["/usr/local/bin/tfchain"]