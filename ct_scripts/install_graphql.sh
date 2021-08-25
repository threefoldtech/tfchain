set -e
sudo /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/crystaluniverse/publishtools/master/scripts/install.sh)"

cd graphql

IP=$(ip -4 addr show eth0 | grep -oP "(?<=inet ).*(?=/)")
echo -e "\nWS_ENDPOINT=ws://$IP:9944" >> .env

yarn

cd indexer
docker-compose build
docker-compose up -d

cd ..

yarn db:up
yarn db:migrate
yarn db:init

yarn processor:migrate

docker-compose build
docker-compose up -d

echo "Graphql up and running"