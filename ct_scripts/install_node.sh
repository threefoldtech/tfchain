#!/bin/bash

set -ex
sudo chown gitpod:gitpod -R /tmp

sudo curl -fsSL https://deb.nodesource.com/setup_14.x | sudo -E bash -
sudo apt-get install -y nodejs
node -v

