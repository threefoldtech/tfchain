# Adding validators to an existing Tfchain network

### Requirements

The most common way for a beginner to run a validator is on a cloud server running Linux. You may choose whatever VPS provider that your prefer, and whatever operating system you are comfortable with.
For this guide we will be using Ubuntu 18.04, but the instructions should be similar for other platforms.

The transactions weights in TFchain were benchmarked on standard hardware. It is recommended that validators run at least the standard hardware in order to ensure they are able to process all blocks in time. The following are not minimum requirements but if you decide to run with less than this beware that you might have performance issue.

Standard Hardware
For the full details of the standard hardware please see here.

- CPU - Intel(R) Core(TM) i7-7700K CPU @ 4.20GHz
- Storage - A NVMe solid state drive. Should be reasonably sized to deal with blockchain growth. Starting around 80GB - 160GB will be okay for the first six months of TFchain, but will need to be re-evaluated every six months.
- Memory - 64GB ECC.

The specs posted above are by no means the minimum specs that you could use when running a validator, however you should be aware that if you are using less you may need to toggle some extra optimizations in order to be equal to other validators that are running the standard.


## Node Prerequisites: Install Rust and Dependencies

Once you choose your cloud service provider and set-up your new server, the first thing you will do is install Rust.

If you have never installed Rust, you should do this first.

If you have already installed Rust, run the following command to make sure you are using the latest version.

`rustup update`

If not, this command will fetch the latest version of Rust and install it.

`curl https://sh.rustup.rs -sSf | sh -s -- -y`

> If you do not have "curl" installed, run "sudo apt install curl"

To configure your shell, run the following command.

`source $HOME/.cargo/env`

Verify your installation.

`rustc --version`

Finally, run this command to install the necessary dependencies for compiling and running the Polkadot node software.

`sudo apt install make clang pkg-config libssl-dev build-essential`

Note - if you are using OSX and you have Homebrew installed, you can issue the following equivalent command INSTEAD of the previous one:

`brew install cmake pkg-config openssl git llvm`

## Install & Configure Network Time Protocol (NTP) Client

NTP is a networking protocol designed to synchronize the clocks of computers over a network. NTP allows you to synchronize the clocks of all the systems within the network. Currently it is required that validators' local clocks stay reasonably in sync, so you should be running NTP or a similar service. You can check whether you have the NTP client by running:

If you are using Ubuntu 18.04 / 19.04, NTP Client should be installed by default.

`timedatectl`

If NTP is installed and running, you should see System clock synchronized: yes (or a similar message). If you do not see it, you can install it by executing:

`sudo apt-get install ntp`

ntpd will be started automatically after install. You can query ntpd for status information to verify that everything is working:

`sudo ntpq -p`

> WARNING: Skipping this can result in the validator node missing block authorship opportunities. If the clock is out of sync (even by a small amount), the blocks the validator produces may not get accepted by the network. This will result in ImOnline heartbeats making it on chain, but zero allocated blocks making it on chain.

## TFchain Binary

You will need to build the tfchain binary from the threefoldtech/tfchain repository on GitHub using the source code available in the 1.0.0 tag.

You should generally use the 1.0.0 tag. You should either review the output from the "git tag" command or visit the Releases to see a list of all the potential 1.0 releases.

Note: If you prefer to use SSH rather than HTTPS, you can replace the first line of the below with git clone git@github.com:threefoldtech/tfchain.git.

```
git clone https://github.com/threefoldtech/tfchain.git
cd substrate-node
git checkout 1.0.0
```

Now build the binary

`cargo build --release`

This step will take a while (generally 10 - 40 minutes, depending on your hardware).

Note if you run into compile errors, you may have to switch to a less recent nightly. This can be done by running:

```
rustup install nightly-2021-06-09
rustup target add wasm32-unknown-unknown --toolchain nightly-2021-06-09
cargo +nightly-2021-06-09 build --release
```

You may also need to run the build more than once.

If you are interested in generating keys locally, you can also install subkey from the same directory. You may then take the generated subkey executable and transfer it to an air-gapped machine for extra security.

```
cargo install --force --git https://github.com/paritytech/substrate subkey
```

## Synchronize Chain Data

Note: By default, Validator nodes are in archive mode. If you've already synced the chain not in archive mode, you must first remove the database with polkadot purge-chain and then ensure that you run Polkadot with the --pruning=archive option.

You may run a validator node in non-archive mode by adding the following flags: --unsafe-pruning --pruning <NUM OF BLOCKS>, a reasonable value being 1000. Note that an archive node and non-archive node's databases are not compatible with each other, and to switch you will need to purge the chain data.

Bootnodes examples:

- Devnet bootnode: `/ip4/185.206.122.7/tcp/30333/p2p/12D3KooWLcMLBg9itjQL1EXsAqkJFPhqESHqJKY7CBKmhhhL8fdp`
- Testnet bootnode: `/ip4/51.68.204.40/tcp/30333/p2p/12D3KooWQv76DZxtZGb7XYXYFGN5xePoDeiMnnp17roJokhsbVSe`
- Mainnet bootnode: `/ip4/185.206.122.83/tcp/30333/p2p/12D3KooWLtsdtQHswnXkLRH7e8vZJHktsh7gfuL5PoADV51JJ6wY`

You can begin syncing your node by running the following command:

`./target/release/tfchain --chain chainspecs/main/chainSpec.json --pruning=archive --bootnodes /ip4/185.206.122.83/tcp/30333/p2p/12D3KooWLtsdtQHswnXkLRH7e8vZJHktsh7gfuL5PoADV51JJ6wY

if you do not want to start in validator mode right away.

```
2022-02-18 15:45:22  Substrate Node    
2022-02-18 15:45:22  ✌️  version 3.0.0-138e5f5-x86_64-linux-gnu    
2022-02-18 15:45:22  ❤️  by Substrate DevHub <https://github.com/substrate-developer-hub>, 2017-2022    
2022-02-18 15:45:22  📋 Chain specification: Tfchain Mainnet    
2022-02-18 15:45:22  🏷 Node name: milky-woman-6216    
2022-02-18 15:45:22  👤 Role: AUTHORITY    
2022-02-18 15:45:22  💾 Database: RocksDb at /home/user/.local/share/tfchain/chains/tfchain_mainnet/db    
2022-02-18 15:45:22  ⛓  Native runtime: substrate-threefold-40 (substrate-threefold-1.tx1.au1)    
2022-02-18 15:45:23  🔨 Initializing Genesis block/state (state: 0x7526…4fb6, header-hash: 0x84cb…5e7b)    
2022-02-18 15:45:23  👴 Loading GRANDPA authority set from genesis on what appears to be first startup.    
2022-02-18 15:45:23  ⏱  Loaded block-time = 6000 milliseconds from genesis on first-launch    
2022-02-18 15:45:23  Using default protocol ID "sup" because none is configured in the chain specs    
2022-02-18 15:45:23  🏷 Local node identity is: 12D3KooWPyq94pmzZ9RQAvrU1SsShjmEVgfgMvyeBifFGf98wdf6    
2022-02-18 15:45:23  📦 Highest known block at #1231
```

Example of node sync:

```
2021-06-17 03:07:39 🔍 Discovered new external address for our node: /ip4/10.26.16.1/tcp/30333/ws/p2p/12D3KooWLtXFWf1oGrnxMGmPKPW54xWCHAXHbFh4Eap6KXmxoi9u
2021-06-17 03:07:40 ⚙️  Syncing 218.8 bps, target=#5553764 (17 peers), best: #24034 (0x08af…dcf5), finalized #23552 (0xd4f0…2642), ⬇ 173.5kiB/s ⬆ 12.7kiB/s
2021-06-17 03:07:45 ⚙️  Syncing 214.8 bps, target=#5553765 (20 peers), best: #25108 (0xb272…e800), finalized #25088 (0x94e6…8a9f), ⬇ 134.3kiB/s ⬆ 7.4kiB/s
2021-06-17 03:07:50 ⚙️  Syncing 214.8 bps, target=#5553766 (21 peers), best: #26182 (0xe7a5…01a2), finalized #26112 (0xcc29…b1a9), ⬇ 5.0kiB/s ⬆ 1.1kiB/s
2021-06-17 03:07:55 ⚙️  Syncing 138.4 bps, target=#5553767 (21 peers), best: #26874 (0xcf4b…6553), finalized #26624 (0x9dd9…27f8), ⬇ 18.9kiB/s ⬆ 2.0kiB/s
2021-06-17 03:08:00 ⚙️  Syncing 37.0 bps, target=#5553768 (22 peers), best: #27059 (0x5b73…6fc9), finalized #26624 (0x9dd9…27f8), ⬇ 14.3kiB/s ⬆ 4.4kiB/s
```

The `--pruning=archive` flag is implied by the `--validator` flag, so it is only required explicitly if you start your node without one of these two options. If you do not set your pruning to archive node, even when not running in validator mode, you will need to re-sync your database when you switch.

If you are interested in determining how much longer you have to go, your server logs (printed to STDOUT from the polkadot process) will tell you the latest block your node has processed and verified. You can then compare that to the current highest block via the [PolkadotJS Block Explorer](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.grid.tf#/explorer).

## Create a Validator object

- dev: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.dev.grid.tf#/explorer
- test: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.test.grid.tf#/explorer
- main: https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ftfchain.grid.tf#/explorer

Open polkadot js link in the browser based on the network you want to validate on.

Browse to accounts and click `Add Account`, generate an account. Take note of the mnemonic.

This account will be your account that manages the Validator and manages your council membership (voting).
Create another account and name it `ANYNAME_STASH`. This account will be your stash account.
Now you should have 2 accounts.

Now go to `Developer` -> `Extrinsicis` and Select your `Stash` account. Now from the left dropdown (modules) search `validator`

![bond](./bond.png)

Select `bond(validator)` and select the target account to be your account that manages the Validator and manages your council membership (voting). (You previously created).

Now click `Submit Transaction`.

To continue click on `bond(v..)` to select another method, in the list search for: `createValidator(...)`.

![create](./create_val.png)

This call needs to be signed with your account that manages the Validator and manages your council membership (voting). (You previously created).

Information needed:

- validator_node_account: Account ID generated from Step 1 (top of this document)
- stash_account: Stash account (previously created)
- description: Reason why I want to become a validator
- tfconnectid: Your Threefold connect name
- info: link to webpage or linked in profile

If all information is filled in correctly. Click on `Submit transaction` and sign. If all goes well, the Council will approve your request.

## Activate validator

If your request is approved by the council AND your tfchain node is fully synced with the network you can activate your validator. This will kickstart block production.

Now go to `Developer` -> `Extrinsicis` and Select your account that manages the Validator and manages your council membership (voting). (You previously created).. Now from the left dropdown (modules) search `validator`.

![activate](./activate.png)

Select `ActivateValidatorNode` and click Submit Transaction. 

## Almost there

Once your node is fully synced, stop the process by pressing Ctrl-C. At your terminal prompt, we will now first insert keys.

### Generate session key

Connect to the new node deployed with polkadot js apps. You will need to install a local version of this application since you will have to connect over a not secured websocket.

Source: https://github.com/polkadot-js/apps

```
git clone git@github.com:polkadot-js/apps.git
yarn
yarn start
```

Browse to http://localhost:3000 and connect to the new node over it's public ip. Make sure to specify the port, like: ws://YOUR_MACHINE_PUBLIC_IP:9944

Go to `Developer` -> `RPC calls` -> `author` -> `rotateKeys`, excecute it and take note of the output.

Go to `accounts` -> `add account` -> use the mnemonic from 'subkey generate' -> give descriptive name -> save

NOTE: !! make sure to use the generated node account, created above, using 'subkey generate' !!

Go to `Extrinsics` -> `session` -> `setKeys` -> (make sure to use the generated node account, created above) -> 

input:
```
keys: the key from rotate keys ouput
proofs: 0
```

### Generate gran/aura key

`subkey generate --scheme sr25519`

Take note of the SS58 address.

Transfer some balance to the new address. (0.1 TFT should be enough). You can transfer the balance to this account from the polkadot UI.

```
./target/release/tfchain key insert --key-type aura --suri "<mnemonic_generated_step_above>" --chain chainspecs/main/chainSpecRaw.json
./target/release/tfchain key insert --key-type gran --suri "<mnemonic_generated_step_above>" --chain chainspecs/main/chainSpecRaw.json
```

Now we can start the node in Validator mode:

```
./target/release/tfchain --chain chainspecs/main/chainSpec.json --validator --name "name on telemetry" --bootnodes /ip4/185.206.122.83/tcp/30333/p2p/12D3KooWLtsdtQHswnXkLRH7e8vZJHktsh7gfuL5PoADV51JJ6wY
```

Your node is now running in validator mode and should be producing blocks