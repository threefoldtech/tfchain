
# Adding Validators to an Existing TFChain Network

This guide provides a step-by-step process to add a validator to the TFChain network.
It covers generating keys using `subkey` via Docker, starting the node, inserting keys, and submitting the necessary proposals for validation.

## Prerequisites

- **Docker Installed**: Ensure Docker is installed and running on your machine.
- **Access to a Server or Machine**: Where you can run the TFChain node and `subkey` via Docker.
- **Polkadot.js Browser Extension**: For managing accounts and signing transactions.
- **Basic Knowledge**: Familiarity with command-line operations and blockchain concepts.

## Hardware

### Requirements

The most common way for a beginner to run a validator is on a cloud server running Linux. You may choose whatever VPS provider that you prefer, and whatever operating system you are comfortable with.
For this guide we will be using Ubuntu 22.04, but the instructions should be similar for other platforms.

The transactions weights in TFchain were benchmarked on standard hardware.
It is recommended that validators run at least the standard hardware in order to ensure they are able to process all blocks in time.
The following are not minimum requirements but if you decide to run with less than this beware that you might have performance issue.

#### Standard Hardware

- CPU
  - x86-64 compatible;
  - Intel Ice Lake, or newer (Xeon or Core series); AMD Zen3, or newer (EPYC or Ryzen);
  - ~4~ 8 physical cores @ 3.4GHz;
  - Simultaneous multithreading disabled (Hyper-Threading on Intel, SMT on AMD);
  - Prefer single-threaded performance over higher cores count. A comparison of single-threaded performance can be found here.
- Storage
  - An NVMe SSD. Should be reasonably sized to deal with blockchain growth. Minimum around 80GB but will need to be re-evaluated every six months.
- Memory
  - ~32~ 64 GB DDR4 ECC.
- System
  - Linux Kernel 5.16 or newer.

The specs posted above are not a hard requirement to run a validator, but are considered best practice. Running a validator is a responsible task; using professional hardware is a must in any way.

## 1. Generate Keys

### 1.1 Generate the Validator Account Key

We'll use `subkey` via Docker to generate a new key pair for your validator account.

```bash
docker run --rm parity/subkey:latest generate --scheme sr25519
```

Take note of the following:

- **Secret Phrase (Mnemonic)**: Securely store this mnemonic; it's crucial for account recovery.
- **Secret Seed**
- **Public Key (hex)**
- **SS58 Address**: This is your validator's account address.

This key will serve as your validator controller account and session key for Aura (validator node/author account). It'll used also to derive the GRANDPA key.

### 1.2 Generate the Node (aka Network) Key

Generate the node key file, which identifies your node in the P2P network.

```bash
docker run --rm parity/subkey:latest generate-node-key > "<node_private_key_file>"
```

This command outputs a **public key** and write **secret seed** (private key) to `<node_private_key_file>` file. Keep the secret seed secure; you'll use it when starting the node.

### 1.3 Derive the GRANDPA Key

Using the same mnemonic from step 1.1, derive the GRANDPA key (Ed25519).

```bash
docker run --rm parity/subkey:latest inspect --scheme ed25519 "mnemonic phrase"
```

Replace `"mnemonic phrase"` with your actual mnemonic enclosed in quotes.

Note down the **Public Key (hex)** for GRANDPA. This key will serve as your session key for GRANDPA.

## 2. Start the Validator Node

You can start the TFChain node either using the binary or via Docker.

### Option A: Using Docker

#### 2.1 Prepare a Persistent Storage Directory

Create a directory on your host machine to store node data (e.g., `/path/to/storage`).

```bash
mkdir -p /path/to/storage
```

#### 2.2 Insert Session Keys

Since the TFChain Docker container doesn't persist keys between runs by default, we'll run temporary containers to insert the keys into the shared volume.

**Aura Key:**

```bash
docker run --rm \
  -v /path/to/storage:/storage \
  ghcr.io/threefoldtech/tfchain:latest \
  key insert \
  --base-path /storage \
  --chain /etc/chainspecs/main/chainSpecRaw.json \
  --key-type aura \
  --suri "<mnemonic phrase>" \
  --scheme sr25519
```

**GRANDPA Key:**

```bash
docker run --rm \
  -v /path/to/storage:/storage \
  ghcr.io/threefoldtech/tfchain:latest \
  key insert \
  --base-path /storage \
  --chain /etc/chainspecs/main/chainSpecRaw.json \
  --key-type gran \
  --suri "<mnemonic phrase>" \
  --scheme ed25519
```

- Ensure the `/path/to/storage` directory is consistent across commands.
- Replace `<mnemonic phrase>` with your mnemonic from step 1.1.

#### 2.3 Start the Node with Warp Sync Enabled

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
  --state-pruning 1000 \
  --offchain-worker Always \
  --sync warp
```

- Replace `<NETWORK>` with the network name you want to join (e.g. `test` or `main`).
- Replace `<node_private_key>` with the node's secret seed from step 1.2.
- Replace `"YourValidatorName"` with your chosen validator name.
- The `--sync warp` flag enables faster synchronization by skipping state transitions from older blocks.

### Option B: Using Kubernetes Deployment

#### 2.1 Create a Custom `values.yaml` File

Create a `values.yaml` file with your configurations for deploying the validator node using Kubernetes.

```yaml
keys:
  - name: aura
    secret: "<mnemonic created by `subkey generate` in step 1>"
  - name: grandpa
    secret: "<mnemonic created by `subkey generate` in step 1>"
  - name: node
    secret: "<node private key generated in step 1>"
volume:
  existingPersistentVolumeClaim: 'your-pvc-name'
  persistentVolume:
    create: false
is_validator: true
disable_offchain_worker: false
chainspec: '/etc/chainspecs/main/chainSpecRaw.json'
sync: warp
ingress:
  enabled: false
```

- Replace `<mnemonic phrase>` with your mnemonic from step 1.1.
- Replace `<node_private_key>` with the node's secret seed from step 1.2.
- Ensure you have a Persistent Volume Claim (PVC) named `'your-pvc-name'` with sufficient storage (e.g., 100Gi).
- Enable warp sync by setting `sync: warp`.

#### 2.2 Deploy the Helm Chart

Install the Helm chart located at `substrate-node/charts/substrate-node` using your custom `values.yaml` file.

```bash
helm install tfchain-validator substrate-node/charts/substrate-node -f values.yaml
```

This will set up the TFChain validator node on your Kubernetes cluster using the configurations specified.

### Option C: Using the TFChain Binary

#### 2.1.1 Install Rust and Dependencies

If you have never installed Rust, you should do this first.

If you have already installed Rust, run the following command to make sure you are using the latest version.

```bash
rustup update
```

If not, this command will fetch the latest version of Rust and install it.

```bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
```

If you do not have "curl" installed, run `sudo apt install curl`

To configure your shell, run the following command.

```bash
source $HOME/.cargo/env
```

Verify your installation.

```bash
rustc --version
```

Finally, run this command to install the necessary dependencies for compiling and running the Polkadot node software.

```bash
sudo apt install make clang pkg-config libssl-dev build-essential
```

Note - if you are using OSX and you have Homebrew installed, you can issue the following equivalent command INSTEAD of the previous one:

```bash
brew install cmake pkg-config openssl git llvm
```

#### 2.1.2 Install & Configure Network Time Protocol (NTP) Client

NTP is a networking protocol designed to synchronize the clocks of computers over a network. NTP allows you to synchronize the clocks of all the systems within the network.
Currently it is required that validators' local clocks stay reasonably in sync, so you should be running NTP or a similar service. You can check whether you have the NTP client by running:

If you are using Ubuntu 18.04 / 19.04, NTP Client should be installed by default.

```bash
timedatectl
```

If NTP is installed and running, you should see System clock synchronized: yes (or a similar message). If you do not see it, you can install it by executing:

```bash
sudo apt-get install ntp
```

ntpd will be started automatically after install. You can query ntpd for status information to verify that everything is working:

```bash
sudo ntpq -p
```

- WARNING: Skipping this can result in the validator node missing block authorship opportunities.
If the clock is out of sync (even by a small amount), the blocks the validator produces may not get accepted by the network.

#### 2.1.3 Compile TFchain Binary

Clone tfchain repo

```bash
git clone https://github.com/threefoldtech/tfchain.git
cd substrate-node
```

Now build the binary

```sh
cargo build --release
```

This step will take a while (generally 10 - 40 minutes, depending on your hardware).
You can find the compiled binary in `./target/release/`

```bash
cd target/release/
```

#### 2.2 Start the Node in Validator Mode

```bash
./tfchain \
  --base-path /storage \
  --chain chainspecs/main/chainSpecRaw.json \
  --validator \
  --node-key-file "<node_private_key_file>" \
  --name "YourValidatorName" \
  --telemetry-url 'wss://shard1.telemetry.tfchain.grid.tf/submit 1' \
  --blocks-pruning archive \
  --state-pruning 1000 \
  --sync warp
```

- Replace `<node_private_key_file>` with the node's secret seed file from step 1.2.
- Replace `"YourValidatorName"` with a name for your node.
- Ensure the `--blocks-pruning archive` and `--state-pruning 1000` flag is used for optimal db size.
- The `--sync warp` flag enables faster synchronization by skipping intermediate state processing.

#### 2.3 Insert Session Keys

Insert the Aura and GRANDPA keys into your node's keystore.

**Aura Key (Sr25519):**

```bash
./tfchain key insert \
  --base-path /storage \
  --chain chainspecs/main/chainSpecRaw.json \
  --key-type aura \
  --suri "<mnemonic phrase>" \
  --scheme sr25519
```

**GRANDPA Key (Ed25519):**

```bash
./tfchain key insert \
  --base-path /storage \
  --chain chainspecs/main/chainSpecRaw.json \
  --key-type gran \
  --suri "<mnemonic phrase>" \
  --scheme ed25519
```

- Replace `<mnemonic phrase>` with your mnemonic from step 1.1.

You can restart your node at this point.

#### 2.4 Managing TFChain with systemd (Optional)

Example systemd file:

```ini
    [Unit]
    Description=TFchain service
    After=network.target
    StartLimitIntervalSec=0

    [Service]
    Type=simple
    Restart=always
    RestartSec=1
    User=user
    ExecStart=/home/user/tfchain/substrate-node/target/release/tfchain --chain /home/user/tfchain/substrate-node/chainspecs/main/chainSpecRaw.json <...>

    [Install]
    WantedBy=multi-user.target
```

- Replace `user` by your username.
- Replace `<...>` by the rest of the tfchain start command args

```bash
sudo vim /etc/systemd/system/tfchain.service
```

then paste the file content, save and exit vim.

Starting service:

```bash
sudo systemctl start tfchain
```

Stopping service:

```bash
sudo systemctl stop tfchain
```

Reload config:

```bash
sudo systemctl stop tfchain
```

Edit File:

```bash
sudo systemctl start tfchain
```

## 3. Synchronize the Node Using Warp Sync

With warp sync enabled, your node will sync much faster by downloading key block headers and skipping state transitions. To monitor the syncing progress, you can view the logs.

```bash
# For Docker:
docker logs -f "<container-name>"

# For binary:
tail -f /storage/logs.txt
```

- Look for log entries indicating the node's **best** and **finalized** block numbers.
- Warp sync should drastically reduce the time to catch up with the current state of the chain.

## 4. Set Session Keys On-Chain

To have your node recognized as a validator, you need to set your session keys on-chain.

### 4.1 Add Validator Account to Polkadot.js Extension

- Open the Polkadot.js browser extension.
- Import your validator controler account using the mnemonic from step 1.1.
- Ensure you have some TFT tokens in this account (0.1 TFT should suffice for transaction fees).

### 4.2 Set Session Keys via PolkadotJS Apps

1. Navigate to [PolkadotJS Apps](https://polkadot.js.org/apps/).
2. Connect to the TFChain network (e.g., wss://tfchain.dev.grid.tf).
3. Go to **Developer → Extrinsics**.
4. Select your validator controler account as the signer.
5. Choose `session` → `setKeys(keys, proof)`.
6. Input your session keys:

   - **keys**: Use previously generated aura and gran hex public keys.
     - **aura**: Manually enter the hex public key of the sr25519 key

     - **gran**: Manually enter the hex public key of the ed25519 key

   - **proof**: Set to `0x00`.

7. Submit the transaction. Once the session keys are set on-chain, your validator will be recognized by the network.

## 5. Submit a Council Motion to Add Validator

TFChain network require a governance proposal to add a validator node.

1. Navigate to **Governance → Council**.
2. Click on **Propose Motion**.
3. Select `validatorSet` → `addValidator(validatorId)`.
4. Input your validator controler account's SS58 address (generated in step 1.1).
5. Submit the proposal.

After submission, inform other council members and request them to vote on the proposal.

## 6. Finalize and Start Validating

Once your session keys are set and the council approves your validator, your node will start participating in block production after 2 era.

### Ensure Node Health

- Keep your node online and synced.
- Monitor logs for any errors or warnings.

---

## References

- **Subkey Utility via Docker**: [Parity Subkey Docker Image](https://hub.docker.com/r/parity/subkey)
- **TFChain Docker Images**: [TFChain Docker Packages](https://github.com/threefoldtech/tfchain/pkgs/container/tfchain)
- **PolkadotJS Apps**: [PolkadotJS Interface](https://polkadot.js.org/apps/)
- **PolkadotJS Extension**: [Browser Extension](https://polkadot.js.org/extension/)
- **Validator Set Pallet**: [Substrate Validator Set Module](https://github.com/gautamdhameja/substrate-validator-set)
- **Council Integration Guide**: [Council Integration](https://github.com/gautamdhameja/substrate-validator-set/blob/master/docs/council-integration.md)
- **Polkadot Validator Guide**: [How to Validate on Polkadot](https://wiki.polkadot.network/docs/maintain-guides-how-to-validate-polkadot)
