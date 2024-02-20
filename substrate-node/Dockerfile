FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /tfchain
ARG FEATURES=default
COPY . .
RUN cargo build --locked --release --features $FEATURES

# ===== SECOND STAGE ======
FROM docker.io/library/ubuntu:22.04
ARG PROFILE=release
# metadata
ARG DOC_URL="https://github.com/threefoldtech/tfchain"

LABEL io.threefoldtech.image.authors="https://www.threefold.tech" \
	io.threefoldtech.image.vendor="Threefold Tech" \
	io.threefoldtech.image.title="threefoldtech/tfchain" \
	io.threefoldtech.image.description="A base image for standard binary distribution" \
	io.threefoldtech.image.source="https://github.com/threefoldtech/tfchain/blob/development/substrate-node/Dockerfile" \
	io.threefoldtech.image.documentation="${DOC_URL}"

# show backtraces
ENV RUST_BACKTRACE 1

COPY --from=builder /tfchain/target/$PROFILE/tfchain /usr/local/bin
COPY --from=builder /tfchain/chainspecs /etc/chainspecs/

RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
	libssl3 ca-certificates && \
	apt-get autoremove -y && apt-get clean && rm -rf /var/lib/apt/lists/* && \
	rm -rf /usr/lib/python* && \
	rm -rf /src && \
	rm -rf /usr/share/man

# checks
RUN ldd /usr/local/bin/tfchain && \
	/usr/local/bin/tfchain --version

EXPOSE 30333 9933 9944 9615
VOLUME ["/tfchain"]

ENTRYPOINT ["/usr/local/bin/tfchain"]
