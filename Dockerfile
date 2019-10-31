# NOTE
# The `proc_macro` feature needed by failure_derive is not available on
# alpine (x86_64-unknown-linux-musl).
# So e.g. `FROM rustlang/rust:nightly-alpine as builder` would not work.
FROM rustlang/rust:nightly-buster-slim as builder

ARG BINARY

WORKDIR /build

RUN set -eux; \
  apt-get update; \
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  ca-certificates \
  make \
  libssl-dev \
  pkg-config \
  \
  libpq-dev=11.5-1+deb10u1

COPY . .

RUN make setup:vendor
RUN make build:release:${BINARY}


FROM debian:buster-slim

ARG BINARY
ENV BINARY ${BINARY}

WORKDIR /app

RUN set -eux; \
  apt-get update; \
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  ca-certificates \
  \
  libpq5=11.5-1+deb10u1

COPY --from=builder /build/target/release/${BINARY} .

CMD /app/${BINARY}
