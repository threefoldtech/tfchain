#!/bin/sh

echo "starting benchmark"

echo "generating pallet-burning"
cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_burning --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-burning/src/weights.rs --template ./frame-weight-template.hbs"
$cmd

echo "generating pallet-dao"
cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_dao --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-dao/src/weights.rs --template ./frame-weight-template.hbs"
$cmd

echo "generating pallet-kvstore"
cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_kvstore --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-kvstore/src/weights.rs --template ./frame-weight-template.hbs"
$cmd

# echo "generating pallet-smart-contract"
# cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_smart_contract --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-smart-contract/src/weights.rs --template ./frame-weight-template.hbs"
# $cmd

# echo "generating pallet-tfgrid weights"
# cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_tfgrid --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-tfgrid/src/weights.rs --template ./frame-weight-template.hbs"
# $cmd

# echo "generating pallet-tft-bridge"
# cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_tft_bridge --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-tft-bridge/src/weights.rs --template ./frame-weight-template.hbs"
# $cmd

echo "generating pallet-tft-price"
cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_tft_price --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-tft-price/src/weights.rs --template ./frame-weight-template.hbs"
$cmd

echo "generating pallet-validator"
cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=pallet_validator --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/pallet-validator/src/weights.rs --template ./frame-weight-template.hbs"
$cmd

echo "generating substrate-validator-set"
cmd="../target/release/tfchain benchmark pallet --chain=dev --pallet=validatorset --extrinsic="*" --steps=50 --repeat=20 --execution=wasm --heap-pages=409 --output ../pallets/substrate-validator-set/src/weights.rs --template ./frame-weight-template.hbs"
$cmd