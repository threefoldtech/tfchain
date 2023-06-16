FROM golang:alpine3.14 as BUILDER

WORKDIR /src

ADD bridge/tfchain_bridge /src/bridge/tfchain_bridge
ADD clients/tfchain-client-go /src/clients/tfchain-client-go

WORKDIR /src

RUN cd /src/bridge/tfchain_bridge &&\
    CGO_ENABLED=0 GOOS=linux go build -ldflags "-w -s -X main.GitCommit=${version} -extldflags '-static'"  -o tfchain_bridge &&\ 
    chmod +x tfchain_bridge

FROM alpine:3.13.5

COPY --from=BUILDER /src/bridge/tfchain_bridge/tfchain_bridge /bin/

ENTRYPOINT [ "/bin/tfchain_bridge" ]
