# Adding validators to an existing Tfchain network

First create a new key:

```sh
subkey generate --scheme sr25519
```

Take note of the SS58 address.

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
  --node-key <node_key>
  --telemetry-url 'wss://shard1.telemetry.tfchain.grid.tf/submit 1'
  --name "somename"
```

Insert Aura and Grandpa key for this new node:

Aura:

```
./tfchain key insert --base-path /storage --chain /etc/chainspecs/dev/chainSpecRaw.json --key-type aura --suri "<mnemonic>" --scheme sr25519
```

Grandpa

```
./tfchain key insert --base-path /storage --chain /etc/chainspecs/dev/chainSpecRaw.json --key-type gran --suri "<mnemonic>" --scheme ed25519
```

Smart contract billing:

```
./tfchain key insert --base-path /storage --chain /etc/chainspecs/dev/chainSpecRaw.json --key-type smct --suri "<mnemonic>" --scheme sr25519
```

### Deploy via Docker

Install Docker and allow a user to run containers:

```sh
usermod -aG docker <username>
```

Disable iptables in Docker. Edit /lib/systemd/system/docker.service and add --iptables=false to ExecStart= ..

```sh
ExecStart=/usr/bin/dockerd -H fd:// --containerd=/run/containerd/containerd.sock --iptables=false
```

Restart Docker

```sh
systemctl daemon-reload
systemctl restart docker
```

Create /storage dir and allow a user to write

```sh
mkdir /storage
```

Start 2 containers to set keys in /storage, these will stop immediately

```sh
docker run -v /storage/:/storage/ dylanverstraete/tfchain:2.1.0 key insert --base-path /storage --chain /etc/chainspecs/dev/chainSpecRaw.json --key-type aura --suri "<mnemonic>" --scheme sr25519
docker run -v /storage/:/storage/ dylanverstraete/tfchain:2.1.0 key insert --base-path /storage --chain /etc/chainspecs/dev/chainSpecRaw.json --key-type gran --suri "<mnemonic>" --scheme ed25519
```

Start another to insert the `smct` key for the billing. Make sure the mneminic is the sr25519 of the validator key (same as the aura key)

```sh
docker run -v /storage/:/storage/ dylanverstraete/tfchain:2.1.0 key insert --base-path /storage --chain /etc/chainspecs/dev/chainSpecRaw.json --key-type smct --suri "<mnemonic>" --scheme sr25519
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
  - name: smct
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

## Allow the node as a validator to the chain

Add the newly generated wallet to the polkadot extension of your browser.

Transfer some balance (0.1 TFT should be enough) to the SS58 address (aura sr25519 key generated using 'subkey generate')

Browse to https://polkadot.js.org/apps and connect to: wss://tfchain.dev.grid.tf

Go to `Extrinsics` -> `session` -> `setKeys` -> (make sure to use the generated node account as the 'selected account', created above) ->

input:

```
aura: <insert the sr25519 Public key (hex) from step (*1)>
grandpa: <insert the ed25519 Public key (hex) from step (*2)>
proofs: 0
```

## Contact admin to insert the key into the validator set

An admin with access to the sudo key can call following extrinsic to insert the new node as a validator. For Devnet, is found in encrypted repo.

Submit the transaction with the SUDO key!

`Developer` -> `Extrinsics` -> `sudo` -> `sudo(call)` -> -> `validatorSet` -> `addValidator` -> `validatorId` must be the account of the node you are adding

validatorID: SS58 encoded address of newly generated keypair (aura sr25519 key generated using 'subkey generate')
