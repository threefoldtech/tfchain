#!/bin/sh
set -ex

rm -rf v
git clone https://github.com/vlang/v
cd v && make && sudo ./v symlink

cd /
export PATH=/v:$PATH

mkdir /root/.vmodules/despiegk
mkdir /root/.vmodules/threefoldtech

git clone https://github.com/threefoldtech/vgrid /root/.vmodules/threefoldtech/vgrid
git clone https://github.com/threefoldtech/rmb /root/.vmodules/threefoldtech/rmb
git clone https://github.com/crystaluniverse/crystallib /root/.vmodules/despiegk/crystallib

cd /root/.vmodules/threefoldtech/rmb/msgbusd && sudo v msgbusd.v && sudo cp msgbusd /usr/local/bin 

