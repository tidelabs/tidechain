# ===== BUILD NODE ======
FROM paritytech/ci-linux:production as builder
LABEL description="This is the build stage for TiDeFi node. Here we create the binary."

ARG CI_JOB_TOKEN

WORKDIR /tidefi
COPY . /tidefi

RUN git config --global credential.helper store && \
	echo "https://gitlab-ci-token:${CI_JOB_TOKEN}@tributary.semantic-network.tech" > ~/.git-credentials

RUN RUST_BACKTRACE=full cargo build --release

# ===== LAUNCH NODE ======
FROM docker.io/library/ubuntu:20.04
LABEL description="This is the 2nd stage: a very small image where we copy the TiDeFi node binary."

# Required for the validators
RUN apt update && apt install -y gpg ca-certificates ubuntu-keyring

# Copy node binary
COPY --from=builder /tidefi/target/release/tidechain /usr/local/bin
# Copy WASM for runtime upgrade (hertel & tidechain)
COPY --from=builder /tidefi/target/release/wbuild/tidechain-runtime/tidechain_runtime.compact.compressed.wasm /data/tidechain_runtime.compact.compressed.wasm
COPY --from=builder /tidefi/target/release/wbuild/hertel-runtime/hertel_runtime.compact.compressed.wasm /data/hertel_runtime.compact.compressed.wasm

RUN useradd -m -u 1000 -U -s /bin/sh -d /tidefi tidefi && \
	mkdir -p /tidefi/.local/share && \
	mkdir -p /data && \
	chown -R tidefi:tidefi /data && \
	ln -s /data /tidefi/.local/share/tidechain && \
	rm -rf /usr/bin /usr/sbin && \
	/usr/local/bin/tidechain --version

USER tidefi

EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/tidechain"]
