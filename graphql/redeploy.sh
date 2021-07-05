#!/bin/bash

docker-compose down

sudo rm -rf /opt/graphqldb
sudo mkdir /opt/graphqldb

yarn

cd node_modules/@subsquid/hydra-typegen && yarn add ws

cd ../../..

yarn build

yarn typegen

yarn workspace sample-mappings install
yarn mappings:build

docker build . -f docker/Dockerfile.builder -t builder
docker build . -f docker/Dockerfile.processor -t processor:latest
docker build . -f docker/Dockerfile.query-node -t query-node:latest

yarn db:up
yarn db:bootstrap
yarn db:init

docker-compose up -d