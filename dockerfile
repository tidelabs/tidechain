# The build container
FROM paritytech/ci-linux:3d4ca6a9-20210708 as builder

WORKDIR /var/www/tidefi-substrate-node

RUN CARGO_HOME=/var/www/tidefi-substrate-node/.cargo

COPY . .

RUN cp -r ./.devcontainer /root/.devcontainer

RUN cargo build --release

# The runtime container
FROM paritytech/ci-linux:3d4ca6a9-20210708

WORKDIR /var/www/tidefi-substrate-node

RUN CARGO_HOME=/var/www/tidefi-substrate-node/.cargo

COPY --from=builder /var/www/tidefi-substrate-node/target/release/ .

CMD ./tidefi-substrate-node --dev --ws-external