FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /tfchain

ARG FEATURES=default

COPY . .

RUN cargo build --locked --release --features $FEATURES

# ===== SECOND STAGE ======

FROM docker.io/library/ubuntu:20.04
LABEL maintainer="dylan@threefold.tech"
LABEL description="This is the 2nd stage: a very small image where we copy the tfchain binary."
ARG PROFILE=release

COPY --from=builder /tfchain/target/$PROFILE/tfchain /usr/local/bin
COPY --from=builder /tfchain/chainspecs /etc/chainspecs/

# checks	
RUN ldd /usr/local/bin/tfchain && \
	/usr/local/bin/tfchain --version

# Install CA certificates not present in base image
RUN apt-get update
RUN apt-get install -y curl
RUN apt-get install -y ca-certificates

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /src && \
	rm -rf /usr/share/man

EXPOSE 30333 9933 9944 9615
VOLUME ["/tfchain"]

ENTRYPOINT ["/usr/local/bin/tfchain"]