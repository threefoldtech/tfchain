set -e
sudo /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/crystaluniverse/publishtools/master/scripts/install.sh)"

cd graphql

IP=$(ip -4 addr show eth0 | grep -oP "(?<=inet ).*(?=/)")
echo -e "\nWS_ENDPOINT=ws://$IP:9944" >> .env

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

