set -e
sudo /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/crystaluniverse/publishtools/master/scripts/install.sh)"

cd cli-tool

yarn

cat readme.md