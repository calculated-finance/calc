# Portable kujirad. 
# Referring to https://github.com/CosmWasm/wasmd
FROM golang:1.18-alpine3.15 AS go-builder
ARG arch=x86_64

RUN set -eux; apk add --no-cache ca-certificates build-base;

RUN apk add git
# NOTE: add these to run with LEDGER_ENABLED=true
#RUN apk add libusb-dev linux-headers
RUN git clone https://github.com/Team-Kujira/core.git /code
WORKDIR /code
RUN git checkout v0.5.0

ADD https://github.com/CosmWasm/wasmvm/releases/download/v1.0.0/libwasmvm_muslc.aarch64.a /lib/libwasmvm_muslc.aarch64.a
ADD https://github.com/CosmWasm/wasmvm/releases/download/v1.0.0/libwasmvm_muslc.x86_64.a /lib/libwasmvm_muslc.x86_64.a
RUN sha256sum /lib/libwasmvm_muslc.aarch64.a | grep 7d2239e9f25e96d0d4daba982ce92367aacf0cbd95d2facb8442268f2b1cc1fc
RUN sha256sum /lib/libwasmvm_muslc.x86_64.a | grep f6282df732a13dec836cda1f399dd874b1e3163504dbd9607c6af915b2740479
RUN cp /lib/libwasmvm_muslc.${arch}.a /lib/libwasmvm_muslc.a

RUN LEDGER_ENABLED=false BUILD_TAGS=muslc LINK_STATICALLY=true make build

FROM alpine:3.15

COPY --from=go-builder /code/build/kujirad /usr/bin/kujirad
COPY setup_and_run_kujirad.sh setup_and_run_kujirad.sh
RUN chmod +x setup_and_run_kujirad.sh
# rest server
EXPOSE 1317
# tendermint p2p
EXPOSE 26656
# tendermint rpc
EXPOSE 26657

CMD ["/usr/bin/kujirad", "version"]`