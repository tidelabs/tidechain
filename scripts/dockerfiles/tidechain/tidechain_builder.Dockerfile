# This is the build stage for Tidechain. Here we create the binary in a temporary image.
FROM docker.io/tidelabs/tidechain-ci:latest as builder

WORKDIR /tidechain
COPY . /tidechain

RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the Tidechain binary."
FROM docker.io/library/ubuntu:20.04

LABEL description="Multistage Docker image for Tidechain" \
	com.semantic-network.image.type="builder" \
	com.semantic-network.image.authors="devops-team@semantic-network.com" \
	com.semantic-network.image.vendor="Semantic Network" \
	com.semantic-network.image.description="Tidechain" \
	com.semantic-network.image.source="https://github.com/tide-labs/tidechain/blob/${VCS_REF}/scripts/dockerfiles/tidechain/tidechain_builder.Dockerfile" \
	com.semantic-network.image.documentation="https://github.com/tide-labs/tidechain/"

COPY --from=builder /tidechain/target/release/tidechain /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /tidechain tidechain && \
	mkdir -p /data /tidechain/.local/share && \
	chown -R tidechain:tidechain /data && \
	ln -s /data /tidechain/.local/share/tidechain && \
	# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
	# check if executable works in this container
	/usr/local/bin/tidechain --version

USER tidechain

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/tidechain"]
