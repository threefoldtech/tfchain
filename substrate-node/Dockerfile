FROM paritytech/ci-linux:c4f0daf3-20220110 as builder

WORKDIR /tfchain

ARG FEATURES=default

COPY . .

RUN cargo build --locked --release --features $FEATURES

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer="dylan@threefold.tech"
LABEL description="This is the 2nd stage: a very small image where we copy the tfchain binary."
ARG PROFILE=release

RUN rm -rf /usr/share/* && \
	mkdir -p /tfchain/.local

COPY --from=builder /tfchain/target/$PROFILE/tfchain /usr/local/bin
COPY --from=builder /tfchain/chainspecs /etc/chainspecs/


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