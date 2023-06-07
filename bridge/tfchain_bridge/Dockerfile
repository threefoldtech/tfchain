FROM golang:alpine3.14 as BUILDER
WORKDIR /opt/tfchain
COPY . .
WORKDIR /opt/tfchain
RUN go build

FROM alpine:3.13.5
COPY --from=BUILDER /opt/tfchain/tfchain_bridge /bin/
ENTRYPOINT [ "/bin/tfchain_bridge" ]
