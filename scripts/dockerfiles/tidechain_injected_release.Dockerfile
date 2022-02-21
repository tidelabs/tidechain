FROM docker.io/library/ubuntu:20.04

# metadata
ARG VCS_REF
ARG BUILD_DATE
ARG TIDECHAIN_VERSION

LABEL com.semantic-network.image.authors="ops@semantic-network.com" \
	com.semantic-network.image.vendor="Semantic Network" \
	com.semantic-network.image.title="${IMAGE_NAME}" \
	com.semantic-network.image.description="Tidechain" \
	com.semantic-network.image.source="https://github.com/tide-labs/tidechain/blob/${VCS_REF}/scripts/docker/tidechain_injected_release.Dockerfile" \
	com.semantic-network.image.revision="${VCS_REF}" \
	com.semantic-network.image.created="${BUILD_DATE}" \
	com.semantic-network.image.documentation="https://github.com/tide-labs/tidechain/"

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y \
	libssl1.1 \
	curl \
	ca-certificates && \
	# add user and link ~/.local/share/tidechain to /data
	useradd -m -u 1000 -U -s /bin/sh -d /tidechain tidechain && \
	mkdir -p /data /tidechain/.local/share && \
	chown -R tidechain:tidechain /data && \
	ln -s /data /tidechain/.local/share/tidechain

# install tidechain
# FIXME: use apt repository
RUN curl -O https://releases.tidefi.io/builds/tidechain/x86_64-debian:stretch/${TIDECHAIN_VERSION}/tidechain.deb && \
	dpkg -i tidechain.deb

# apt cleanup
RUN apt-get remove -y curl && \
	apt-get autoremove -y && \
	apt-get clean && \
	find /var/lib/apt/lists/ -type f -not -name lock -delete

USER tidechain

EXPOSE 30333 9933 9944
VOLUME ["/tidechain"]

# check if executable works in this container
RUN /usr/bin/tidechain --version

ENTRYPOINT ["/usr/bin/tidechain"]
