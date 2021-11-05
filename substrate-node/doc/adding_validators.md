# Adding validators to an existing Tfchain network

First create a new key:

```
subkey generate --scheme sr25519
```

Take note of the SS58 address.

Transfer some balance to the new address. (0.1 TFT should be enough)

## Start a new node in validator mode with new key

```
./tfchain \
  --base-path /storage \
  --chain chainspecs/test/chainSpecRaw.json \
  --validator \
  --bootnodes IP_OF_BOOTNODE
```

Insert Aura and Grandpa key for this new node.

## Insert a key in the session

Connect to the new node deployed with polkadot js apps. You will need to install a local version of this application since you will have to connect over a not secured websocket.

```
git clone git@github.com:polkadot-js/apps.git
yarn
yarn start
```

Browse to http://localhost:3000 and connect to the new node over it's public ip.

Go to `rpc` -> `session` -> `rotateKeys`, excecute it and take note of the output.

On the same new node, go to `extrinsics` -> `session` -> `setKeys`, Make sure you execute it this with the newly generated keypair.

input:
```
keys: the key from rotate keys ouput
proofs: 0
```

## Contact admin to insert the key into the validator set

An admin with access to the sudo key can call following extrinsic to insert the new node as a validator:

`extrinsics` -> `validatorSet` -> `addValidator` 

validatorID: SS58 encoded address of newly generated keypair (see step 1)