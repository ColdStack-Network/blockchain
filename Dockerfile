# syntax=docker/dockerfile:experimental

FROM paritytech/ci-linux:production as builder
LABEL description="This is the build stage for Coldstack. Here we create the binary."

ARG PROFILE=release
WORKDIR /coldstack

COPY . /coldstack

RUN cargo build --$PROFILE

# ===== SECOND STAGE ======

FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the Polkadot binary."
ARG PROFILE=release
COPY --from=builder /coldstack/target/$PROFILE/node-template /usr/local/bin/coldstack

COPY chainspec/staging/chainspecRaw.json /chainspec/staging.json

RUN useradd -m -u 1000 -U -s /bin/sh -d /coldstack coldstack && \
	mkdir -p /coldstack/.local/share && \
	mkdir /data && \
	chown -R coldstack:coldstack /data && \
	ln -s /data /coldstack/.local/share/coldstack && \
	rm -rf /usr/bin /usr/sbin

USER coldstack
EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/coldstack"]
