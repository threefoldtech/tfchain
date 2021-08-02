set -ex
cd substrate-node

if ! command -v rustc &> /dev/null
then
    bash ../ct_scripts/install_rust.sh
    exit
fi

cargo build --release

./target/release/tfchain --ws-external --dev --alice
