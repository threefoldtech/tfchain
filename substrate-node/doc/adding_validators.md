# Adding validators to an existing Tfchain network

First create a new key:

```
subkey generate --scheme sr25519
```

Take note of the Public key (hex)

## Start a new node in validator mode with new key

```
./tfchain \
  --base-path /storage \
  --chain chainspecs/test/chainSpecRaw.json \
  --validator \
  --bootnodes IP_OF_BOOTNODE
```

Insert Aura and Grandpa key for this new node.

## Insert the Public Key in the session

Connect to any public node and go to `extrinsics` -> `session` -> `setKeys`

```
keys: generated public key (see step 1)
proof: 0x
```

This extrinsic must be signed by the newly generated key.

## Contact admin to insert the key into the validator set

An admin with access to the sudo key can call following extrinsic to insert the new node as a validator:

`extrinsics` -> `validatorSet` -> `addValidator` 

validatorID: SS58 encoded address of newly generated keypair (see step 1)