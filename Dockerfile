ARG VARNISHOTEL_VARNISH_VERSION=7.7.1
ARG VARNISHLOGJSON_REVISION=main

FROM varnish:${VARNISHOTEL_VARNISH_VERSION} AS builder

ENV PATH="/root/.cargo/bin:${PATH}"

USER root

RUN set -ex; \
    apt-get update; \
    apt-get install -qq \
      git \
      cmake \
      docutils \
      jq \
      libcjson1 \
      libcjson-dev \
      pkg-config \
      curl

RUN set -ex; \
    git clone https://github.com/varnish/varnishlog-json.git /tmp/varnishlog-json; \
    cd /tmp/varnishlog-json; \
    git checkout ${VARNISHLOGJSON_REVISION}; \
    cmake -B build; \
    cmake --build build/; \
    ctest --test-dir build/; \
    cmake --install build/; \
    install build/varnishlog-json /usr/sbin/varnishlogjson

RUN set -ex; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y;

WORKDIR /src/varnishotel
COPY . .

RUN set -ex; \
    cargo build --release; \
    install target/release/varnishotel /usr/sbin/varnishotel

FROM varnish:${VARNISHOTEL_VARNISH_VERSION}

USER root

RUN set -ex; \
    apt-get update; \
    apt-get install -y \
      libcjson-dev; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*;

COPY --from=builder /usr/sbin/varnishlogjson /usr/sbin/varnishlogjson
COPY --from=builder /usr/sbin/varnishotel /usr/sbin/varnishotel

USER varnish

ENTRYPOINT [ "/usr/sbin/varnishotel" ]