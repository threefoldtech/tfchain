set -e
sudo /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/crystaluniverse/publishtools/master/scripts/install.sh)"

#get the dependencies for the cmd line client
cd cli-tool && sudo yarn



