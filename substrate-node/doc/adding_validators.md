# Adding validators to an existing Tfchain network

First create a new key:

```sh
subkey generate --scheme sr25519
```

Take note of the SS58 address.

Transfer some balance to the new address. (0.1 TFT should be enough)

Create a node key

```sh
subkey generate-node-key
```

This will write the node adress on std err followed by the node private key on stdout.

## Start node

### using the tfchain binary

#### Start a new node in validator mode with new key

```sh
./tfchain \
  --base-path /storage \
  --chain chainspecs/main/chainSpecRaw.json \
  --validator \
  --bootnodes IP_OF_BOOTNODE
  --node-key <node_key>
  --telemetry-url 'wss://shard1.telemetry.tfchain.grid.tf/submit 1'
```

Insert Aura and Grandpa key for this new node.

#### Insert a key in the session

Connect to the new node deployed with polkadot js apps. You will need to install a local version of this application since you will have to connect over an unsecured websocket.

```sh
git clone git@github.com:polkadot-js/apps.git
yarn
yarn start
```

Browse to <http://localhost:3000> and connect to the new node over it's public ip.

Go to `rpc` -> `session` -> `rotateKeys`, execute it and take note of the output.

On the same new node, go to `extrinsics` -> `session` -> `setKeys`, Make sure you execute it this with the newly generated keypair.

input:

```log
keys: the key from rotate keys ouput
proofs: 0
```

### using kubernetes

create a custom values.yaml file.

Add the keys:

```yaml
keys:
  - name: aura
    secret: "<mnemonic created by `subkey generate` in step 1>"
  - name: grandpa
    secret: "<mnemonic created by `subkey generate` in step 1>"
  - name: node
    secret: <node private key generated in step 1>
```

Create a Peristent volume claim of 100Gi and assign it using

```yaml
volume:
  existingpersistentVolumeClaim: "your-pvc-name"
  persistentVolume:
    create: false
```

Make sure to set the following values too:

```yaml
is_validator: true
disable_offchain_worker: true
chainspec: "/etc/chainspecs/main/chainSpecRaw.json"
boot_node: "/ip4/185.206.122.83/tcp/30333/p2p/12D3KooWLtsdtQHswnXkLRH7e8vZJHktsh7gfuL5PoADV51JJ6wY"
```

for now, disable the ingress:

```yaml
ingress:
  enabled: false
```

install the helm-chart located at `substrate-node/charts/substrate-node` in this repository using the custom values.yaml file.

## Sync

Wait until you are synced with the network, you can see the last synced (best) and finalized blocks in the logs and highest block on the network as target.

## Contact admin to insert the key into the validator set

An admin with access to the sudo key can call following extrinsic to insert the new node as a validator:

`extrinsics` -> `validatorSet` -> `addValidator`

validatorID: SS58 encoded address of newly generated keypair (see step 1)
