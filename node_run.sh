set -ex
cd substrate-node
bash ../ct_scripts/install_rust.sh
cargo build "--release"