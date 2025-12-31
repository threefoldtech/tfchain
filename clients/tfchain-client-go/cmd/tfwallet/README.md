# tfwallet

A simple CLI tool to interact with TFChain wallets.

## Build

```bash
./build.sh
```

## Usage

Set your private key via environment variable (mnemonic or hex seed):

```bash
export TFCHAIN_KEY="your mnemonic phrase here"
```

### Show wallet info

```bash
./tfwallet
```

Output:
```toml
address = "5EFH3jsZyriLXZ13GtyeBPKoaEUvBYpnpvaMgk9bZ9JPfFvJ"
twin_id = 2
is_hoster = false

[balance]
free = "528404.4863972"
reserved = "0.0000000"
free_utft = "5284044863972"
```

For hosters, it also shows farm and node information.

### Send TFT

```bash
./tfwallet sendto <address> tft <amount>
```

Example:
```bash
./tfwallet sendto 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY tft 10.5
```

Output:
```toml
transferred = "10.5"
to = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
amount_utft = 105000000
```

## Networks

The tool connects to TFChain mainnet (`wss://tfchain.grid.tf/ws`).
