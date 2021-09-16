# ===== BUILD NODE ======
FROM paritytech/ci-linux:9575dfcd-20210729 as node-builder
LABEL description="This is the build stage for TiDeFi node. Here we create the binary."

ARG CI_JOB_TOKEN

WORKDIR /tidefi
COPY . /tidefi

RUN git config --global credential.helper store && \
	echo "https://gitlab-ci-token:${CI_JOB_TOKEN}@tributary.semantic-network.tech" > ~/.git-credentials

RUN RUST_BACKTRACE=full cargo build -p tidefi-substrate-node --release

# ===== BUILD CONTRACT ======
FROM paritytech/contracts-ci-linux:2b738e34-20210829 as contract-builder
LABEL description="This is the build stage for TiDeFi contract. Here we create the contract and the manager."

ARG CI_JOB_TOKEN

WORKDIR /tidefi
COPY . /tidefi

RUN git config --global credential.helper store && \
	echo "https://gitlab-ci-token:${CI_JOB_TOKEN}@tributary.semantic-network.tech" > ~/.git-credentials

RUN cd /tidefi/substrate-contract; cargo +nightly contract build
RUN cd /tidefi/utils/contract-manager; cargo build --release

# ===== LAUNCH NODE AND UPLOAD CONTRACT ======
FROM debian:buster-slim
LABEL description="This is the 3rd stage: a very small image where we copy the TiDeFi node binary."
COPY --from=node-builder /tidefi/target/release/tidefi-substrate-node /usr/local/bin
COPY --from=contract-builder /tidefi/utils/contract-manager/target/release/tidefi-contract-manager /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /tidefi tidefi && \
	mkdir -p /tidefi/.local/share && \
	mkdir -p /data && \
	chown -R tidefi:tidefi /data && \
	ln -s /data /tidefi/.local/share/tidefi-substrate-node && \
	rm -rf /usr/bin /usr/sbin

COPY --from=contract-builder /tidefi/target/ink/tidefi_contract_wrapr/tidefi_wrapr.wasm /tidefi/tidefi_wrapr.wasm
COPY --from=contract-builder /tidefi/target/ink/tidefi_contract_wrapr/metadata.json /tidefi/tidefi_wrapr_metadata.json

USER tidefi
EXPOSE 3000 30333 9933 9944
VOLUME ["/data"]

ENV TIDEFI_NODE=/usr/local/bin/tidefi-substrate-node
ENV TIDEFI_CONTRACT=/tidefi/tidefi_wrapr.wasm
ENV TIDEFI_CONTRACT_META=/tidefi/tidefi_wrapr_metadata.json

CMD ["/usr/local/bin/tidefi-contract-manager"]
