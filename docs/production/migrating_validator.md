# Migrating a Validator to a New Machine

This guide covers how to migrate an existing TFChain validator node from one machine to another without losing your validator status or causing equivocation.

## Important Warnings

- **Never run two validator nodes with the same session keys at the same time.** Running duplicate validators causes equivocation (signing conflicting blocks), which the network detects via `check_for_equivocation` in the Aura consensus and GRANDPA's `submit_report_equivocation_unsigned_extrinsic`. This can lead to loss of reputation and potential removal from the validator set.
- **The old node must be fully stopped before the new node starts validating.** There must be zero overlap.

## Overview

A validator migration involves:

1. Preparing the new machine
2. Syncing the new node (non-validator mode)
3. Transferring or re-inserting session keys
4. Stopping the old validator
5. Starting the new node in validator mode
6. Verifying the migration

## Prerequisites

- Access to both old and new machines
- The new machine meets the [hardware requirements](../misc/adding_validators.md#requirements)
- Your session key mnemonic (secret phrase) backed up from the original setup
- Your node key (network key) file if migrating a boot node

## 1. Prepare the New Machine

### 1.1 Install Dependencies

```bash
# Install system dependencies (not needed if using Docker)
sudo apt install make clang pkg-config libssl-dev build-essential

# Install NTP (required for validators)
# Docker containers share the host's kernel clock, so NTP must be
# configured on the host machine even when running via Docker.
sudo apt-get install ntp
timedatectl  # Verify "System clock synchronized: yes"
```

### 1.2 Get the TFChain Binary

Either download the binary from a [release](https://github.com/threefoldtech/tfchain/releases), or build from source:

```bash
git clone https://github.com/threefoldtech/tfchain.git
cd tfchain/substrate-node
cargo build --release
# Binary is at ./target/release/tfchain
```

Or use Docker:

```bash
docker pull ghcr.io/threefoldtech/tfchain:latest
```

## 2. Sync the New Node

Start the node in **non-validator mode** first to sync the chain without producing blocks. Use `--sync warp` for fastest sync.

### Using the binary:

```bash
./tfchain \
  --base-path /storage \
  --chain chainspecs/<NETWORK>/chainSpecRaw.json \
  --name "YourNode-syncing" \
  --blocks-pruning archive \
  --state-pruning 1000 \
  --sync warp
```

### Using Docker:

```bash
docker run -d --name tfchain-sync \
  -v /path/to/storage:/storage \
  ghcr.io/threefoldtech/tfchain:latest \
  --base-path /storage \
  --chain /etc/chainspecs/<NETWORK>/chainSpecRaw.json \
  --name "YourNode-syncing" \
  --blocks-pruning archive \
  --state-pruning 1000 \
  --sync warp
```

Replace `<NETWORK>` with your target network (`main`, `test`, `qanet`, or `dev`).

Monitor sync progress in the logs. Wait until the node is fully synced (best block matches finalized block on the network).

**Stop the syncing node before proceeding to the next step.**

## 3. Transfer Session Keys to the New Machine

TFChain validators use two session keys:

| Key Type | Scheme | Key Type ID |
| :------- | :----- | :---------- |
| AURA | sr25519 | `aura` |
| GRANDPA | ed25519 | `gran` |

There are three ways to get session keys onto the new machine. All three result in the **same key files** in the keystore. Since the keys are the same, **no on-chain `setKeys` transaction is needed** after migration.

### Option A: Re-insert Keys Using the Mnemonic (Recommended)

If you have the mnemonic (secret phrase) backed up from the original key generation, you can directly re-insert the same keys on the new machine. This is the cleanest and most reliable method.

The `--suri` parameter accepts a mnemonic phrase directly:

**Using the binary:**

```bash
# Insert AURA key
./tfchain key insert \
  --base-path /storage \
  --chain chainspecs/<NETWORK>/chainSpecRaw.json \
  --key-type aura \
  --suri "<your mnemonic phrase>" \
  --scheme sr25519

# Insert GRANDPA key (same mnemonic, different scheme)
./tfchain key insert \
  --base-path /storage \
  --chain chainspecs/<NETWORK>/chainSpecRaw.json \
  --key-type gran \
  --suri "<your mnemonic phrase>" \
  --scheme ed25519
```

**Using Docker:**

```bash
# Insert AURA key
docker run --rm \
  -v /path/to/storage:/storage \
  ghcr.io/threefoldtech/tfchain:latest \
  key insert \
  --base-path /storage \
  --chain /etc/chainspecs/<NETWORK>/chainSpecRaw.json \
  --key-type aura \
  --suri "<your mnemonic phrase>" \
  --scheme sr25519

# Insert GRANDPA key
docker run --rm \
  -v /path/to/storage:/storage \
  ghcr.io/threefoldtech/tfchain:latest \
  key insert \
  --base-path /storage \
  --chain /etc/chainspecs/<NETWORK>/chainSpecRaw.json \
  --key-type gran \
  --suri "<your mnemonic phrase>" \
  --scheme ed25519
```

Both AURA and GRANDPA keys are derived from the **same mnemonic** but use different cryptographic schemes (sr25519 vs ed25519), which produces different key pairs.

The `key insert` command creates key files in the keystore directory at `<base-path>/chains/<chain_id>/keystore/` (or at `--keystore-path` if specified).

### Option B: Copy the Keystore Directory

You can directly copy the keystore files from the old machine to the new one. This is straightforward but requires access to the old machine's filesystem.

**Keystore location** (default): `<base-path>/chains/<chain_id>/keystore/`

For example, with `--base-path /storage` on mainnet: `/storage/chains/tfchain_mainnet/keystore/`

The chain ID depends on your network:

| Network | Chain ID |
| :------ | :------- |
| Mainnet | `tfchain_mainnet` |
| Testnet | `tfchain_testnet` |
| QA Net | `tfchain_qa_net` |
| Devnet | `tfchain_devnet` |

If using a custom `--keystore-path` (e.g., in Kubernetes deployments), the keys are at that path instead.

**Steps:**

```bash
# On the OLD machine - stop the validator first!
sudo systemctl stop tfchain  # or docker stop <container>

# Copy the keystore to the new machine
scp -r /storage/chains/tfchain_mainnet/keystore/ user@new-machine:/storage/chains/tfchain_mainnet/keystore/
```

**What's inside the keystore:**

Each key is stored as a single file:
- **File name**: hex-encoded key type ID + hex-encoded public key (no `0x` prefix)
- **File content**: the mnemonic (secret phrase)

For example, an AURA key file would be named:
```
61757261<hex_public_key>
```
Where `61757261` is the hex encoding of `aura`.

A GRANDPA key file would be named:
```
6772616E<hex_public_key>
```
Where `6772616E` is the hex encoding of `gran`.

You should see exactly **2 files** in the keystore (one for aura, one for gran).

### Option C: Rotate Keys (Generates New Keys)

If you have lost your mnemonic and cannot access the old keystore, you must generate **new** session keys. This requires an additional on-chain `setKeys` transaction.

**Start the new node** (temporarily with `--rpc-methods Unsafe` on localhost only):

```bash
./tfchain \
  --base-path /storage \
  --chain chainspecs/<NETWORK>/chainSpecRaw.json \
  --validator \
  --rpc-methods Unsafe \
  --name "YourValidatorName" \
  --blocks-pruning archive \
  --state-pruning 1000
```

**Rotate keys via RPC:**

```bash
echo '{"id":1,"jsonrpc":"2.0","method":"author_rotateKeys","params":[]}' | \
  websocat -n1 -B 99999999 ws://127.0.0.1:9944
```

The output `result` field contains the concatenated hex public keys for the new session keys.

**Submit `setKeys` on-chain:**

1. Go to [PolkadotJS Apps](https://polkadot.js.org/apps/) connected to TFChain
2. Navigate to **Developer -> Extrinsics**
3. Select your validator controller account
4. Choose `session` -> `setKeys(keys, proof)`
5. Enter the new aura and grandpa hex public keys, proof: `0x00`
6. Submit the transaction

**Then restart the node without `--rpc-methods Unsafe`.**

> Note: The new keys will become active after 2 sessions. Your validator may miss blocks during this transition.

## 4. Stop the Old Validator

**This step is critical.** The old validator must be completely stopped before the new one starts in validator mode.

```bash
# Using systemd
sudo systemctl stop tfchain

# Using Docker
docker stop tfchain-validator
```

Verify it is stopped:

```bash
# Check no tfchain process is running
ps aux | grep tfchain

# For Docker
docker ps | grep tfchain
```

## 5. Start the New Node in Validator Mode

Once the old node is stopped and session keys are in place on the new machine:

### Using the binary:

```bash
./tfchain \
  --base-path /storage \
  --chain chainspecs/<NETWORK>/chainSpecRaw.json \
  --validator \
  --node-key-file "<node_private_key_file>" \
  --name "YourValidatorName" \
  --telemetry-url 'wss://shard1.telemetry.tfchain.grid.tf/submit 1' \
  --blocks-pruning archive \
  --state-pruning 1000
```

### Using Docker:

```bash
docker run -d --name tfchain-validator \
  -v /path/to/storage:/storage \
  ghcr.io/threefoldtech/tfchain:latest \
  --base-path /storage \
  --chain /etc/chainspecs/<NETWORK>/chainSpecRaw.json \
  --validator \
  --node-key "<node_private_key>" \
  --name "YourValidatorName" \
  --telemetry-url 'wss://shard1.telemetry.tfchain.grid.tf/submit 1' \
  --blocks-pruning archive \
  --state-pruning 1000
```

Consider setting up a [systemd service](../misc/adding_validators.md#24-managing-tfchain-with-systemd-optional) to auto-restart the node on failure or reboot.

## 6. Verify the Migration

### Check logs

Look for block authoring messages in the logs. You should see your node producing blocks:

```bash
# Binary
journalctl -u tfchain -f

# Docker
docker logs -f tfchain-validator
```

### Check on-chain

Using [PolkadotJS Apps](https://polkadot.js.org/apps/) connected to TFChain:

1. **Verify your validator is in the active set**: Query `validatorSet -> validators` and confirm your controller account SS58 address is listed.
2. **Verify session keys are registered**: Query `session -> queuedKeys` and confirm your account has the correct aura and grandpa public keys.

## 7. Clean Up the Old Machine

After confirming the new validator is working correctly:

```bash
# Remove the old keystore (contains mnemonics!)
rm -rf /storage/chains/<chain_id>/keystore/

# Optionally remove all chain data
rm -rf /storage/
```

## Additional: Migrating a Boot Node

If your validator also serves as a **boot node** (listed in the chain spec), you must also migrate the **node key** (network key). The node key determines the node's PeerId, which is referenced in the chain spec's `bootNodes` list.

1. Copy the node key file from the old machine to the new one
2. Use `--node-key-file` to point to it when starting the new node
3. Update DNS records to point the boot node domain (e.g., `01.bootnode.main.grid.tf`) to the new machine's IP address

If you lose the node key, your node will get a new PeerId and the old boot node address in the chain spec will stop working. Other nodes will still function if other boot nodes are available, but the chain spec should be updated.

## Quick Reference: Migration Checklist

- [ ] New machine meets hardware requirements
- [ ] NTP is configured and synchronized
- [ ] TFChain binary or Docker image installed
- [ ] New node synced to latest block
- [ ] Session keys transferred (mnemonic re-insert, keystore copy, or key rotation)
- [ ] Old validator completely stopped
- [ ] New validator started with `--validator` flag
- [ ] Logs show block production
- [ ] On-chain validator status confirmed
- [ ] Old machine keystore securely wiped
- [ ] (If boot node) Node key migrated and DNS updated
