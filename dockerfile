# The build container
FROM rustlang/rust:nightly-alpine3.12

COPY . .

CMD bin/sh