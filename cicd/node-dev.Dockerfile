# ===== BUILD NODE ======
FROM paritytech/ci-linux:9575dfcd-20210729 as builder
LABEL description="This is the build stage for TiDeFi node. Here we create the binary."

ARG CI_JOB_TOKEN

WORKDIR /tidefi
COPY . /tidefi

RUN git config --global credential.helper store && \
	echo "https://gitlab-ci-token:${CI_JOB_TOKEN}@tributary.semantic-network.tech" > ~/.git-credentials

RUN RUST_BACKTRACE=full cargo build -p tidefi-node --release

# ===== LAUNCH NODE ======
FROM debian:buster-slim
LABEL description="This is the 2nd stage: a very small image where we copy the TiDeFi node binary."
COPY --from=builder /tidefi/target/release/tidefi-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /tidefi tidefi && \
	mkdir -p /tidefi/.local/share && \
	mkdir -p /data && \
	chown -R tidefi:tidefi /data && \
	ln -s /data /tidefi/.local/share/tidefi-substrate-node && \
	rm -rf /usr/bin /usr/sbin

USER tidefi
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/tidefi-node"]
