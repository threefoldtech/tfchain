# Bridge Local Development Setup

This document describes how to run a complete local bridge environment for development and
testing — single-validator and multi-validator — using the Make targets provided in the
repository root.

---

## Prerequisites

| Tool | Minimum version | Notes |
|---|---|---|
| Go | 1.21 | `go version` |
| Rust + Cargo | stable | via [rustup](https://rustup.rs/) |
| Node.js + npm | 18 | `node --version` |
| Internet | — | Stellar testnet (Friendbot + Horizon) |

---

## Quick Start

### Single-validator

```bash
make bridge-dev
```

That's it. On first run this builds TFChain (Rust, ~20–40 min). Every subsequent run reuses
the existing binary and takes ~1 min.

### Multi-validator (3 daemons, 2-of-3 threshold)

```bash
make bridge-mv-dev
```

Same one-shot command. Spins up 3 bridge daemons (Alice, Bob, Charlie), configures the bridge
Stellar account as a 2-of-3 multi-sig, and runs the full MV test suite.

---

## What `make bridge-dev` Does

| Step | Target | What happens |
|---|---|---|
| 1 | `bridge-clean` | Kill any running bridge/TFChain processes, delete persistency files and logs |
| 2 | `bridge-build` | `go build` the bridge binary (fast, ~5s) |
| 3 | _(auto)_ | Build TFChain node if binary missing (`substrate-node/target/release/tfchain`) |
| 4 | `bridge-accounts` | Generate fresh Stellar keypairs; fund via Friendbot; create TFT trustlines; issue TFT to bridge via `path_payment_strict_send` and to user via `payment`; write `/tmp/bridge_local_env.sh` |
| 5 | `bridge-tfchain-start` | Start TFChain `--dev --tmp`; poll WS until node is ready |
| 6 | `bridge-setup` | Register Alice as bridge validator; set bridge wallet, fee account, deposit/withdraw fees via sudo |
| 7 | `bridge-start` | Start bridge daemon; wait for `bridge_started` log entry |
| 8 | `bridge-test` | Run 4-scenario E2E test suite |

TFChain is built once via Make's file-dependency model. If `substrate-node/target/release/tfchain`
already exists, the Rust build step is skipped entirely.

---

## Individual Targets

```bash
# Build
make bridge-build           # Go build only (fast)
make bridge-build-tfchain   # Rust build (slow, one-time)

# Environment lifecycle
make bridge-accounts        # (Re)generate Stellar accounts → /tmp/bridge_local_env.sh
make bridge-tfchain-start   # Start TFChain dev node
make bridge-setup           # Configure bridge pallet on TFChain
make bridge-start           # Start bridge daemon
make bridge-stop            # Stop bridge daemon
make bridge-tfchain-stop    # Stop TFChain node
make bridge-clean           # Stop everything + delete all local state

# Testing
make bridge-test            # Run E2E tests against a running environment
```

### Configuration overrides

All targets accept environment variable overrides:

```bash
TFCHAIN_URL=ws://localhost:9944 \   # default
BRIDGE_TFT_FLOAT=20000 \            # TFT issued to bridge wallet
USER_TFT_AMOUNT=1000 \              # TFT issued to test user
DEPOSIT_FEE=10000000 \              # 1 TFT (7 decimal places)
WITHDRAW_FEE=10000000 \             # 1 TFT
BRIDGE_ENV_FILE=/tmp/bridge_local_env.sh \
  make bridge-dev
```

---

## Account Sharing Between Steps

All scripts share account details via a single env file written by `make bridge-accounts`:

```
/tmp/bridge_local_env.sh      # single-validator
/tmp/bridge_mv_env.sh         # multi-validator
```

Each subsequent script (`bridge_setup.js`, `bridge_tests.js`, etc.) calls `loadEnv()` at
startup to read this file into `process.env`. The Makefile shell targets source it for
`BRIDGE_SECRET`, `BRIDGE_ADDRESS`, etc.

Re-running `make bridge-accounts` generates fresh Stellar keypairs and invalidates the current
environment — you would need to re-run `bridge-setup` and `bridge-start` as well. The
`make bridge-clean` + `make bridge-dev` cycle handles this automatically.

---

## Logs

```bash
tail -f /tmp/bridge_local.log    # bridge daemon
tail -f /tmp/tfchain_local.log   # TFChain node

# Multi-validator
tail -f /tmp/bridge_mv_1.log     # Val1 (Alice)
tail -f /tmp/bridge_mv_2.log     # Val2 (Bob)
tail -f /tmp/bridge_mv_3.log     # Val3 (Charlie)
```

---

## Tests

### Single-validator tests (`make bridge-test`)

| Test | Description | Expected outcome |
|---|---|---|
| 1 | Normal withdraw | Swap 2 TFT on TFChain → receive 1 TFT on Stellar (1 TFT fee) |
| 2 | Batch withdraws | 5 simultaneous swaps in one block → all 5 delivered |
| 3 | Bad deposit | Send TFT to bridge without memo → full refund on Stellar |
| 4 | Crash recovery | SIGKILL bridge mid-withdraw → restart → delivery completes |

### Multi-validator tests (`make bridge-mv-test`)

| Test | Description | Expected outcome |
|---|---|---|
| MV1 | Normal withdraw | 3 validators, threshold=2; 1 TFT delivered |
| MV2 | Deposit/mint | All 3 validators propose; mint threshold met |
| MV3 | Bad deposit | All 3 detect bad deposit; full refund delivered |
| MV4 | Validator offline | Val3 killed; Val1+Val2 meet threshold=2; refund works; Val3 restarted after |
| MV5 | Batch withdraws | 3 simultaneous burns; all 3 delivered (uses expiry recovery if sequence collision) |

The test runner exits non-zero on any failure, making it composable with CI.

---

## Multi-validator Setup Details

### What `make bridge-mv-dev` does

| Step | Target | What happens |
|---|---|---|
| 1 | `bridge-mv-clean` | Kill all 3 daemons, delete MV persistency files and logs |
| 2 | `bridge-build` | Build bridge binary |
| 3 | _(auto)_ | Build TFChain if binary missing |
| 4 | `bridge-mv-accounts` | Generate 4 keypairs (val1=bridge, val2, val3, user); fund via Friendbot; create trustlines; fund bridge via `path_payment_strict_send`; configure bridge as 2-of-3 multi-sig (val1 master key, val2+val3 added as signers, thresholds: low=1, med=2, high=3); write `/tmp/bridge_mv_env.sh` |
| 5 | `bridge-tfchain-start` | Start TFChain dev node |
| 6 | `bridge-mv-setup` | Create twins for Alice, Bob, Charlie; register all 3 as validators; set bridge wallet, fee account, fees |
| 7 | `bridge-mv-start` | Start 3 bridge daemons; wait for all 3 to log `bridge_started` |
| 8 | `bridge-mv-test` | Run MV1–MV5 test suite |

### Multi-sig architecture

```
Bridge Stellar account = Val1's keypair (master key, weight=1)
Val2 keypair added as signer (weight=1)
Val3 keypair added as signer (weight=1)

Thresholds:
  low  = 1  (any 1 of 3 can change account options)
  med  = 2  (any 2 of 3 must sign TFT payment transactions)
  high = 3  (all 3 must sign for account deletion etc.)
```

Each bridge daemon signs with its own validator Stellar key and stores its signature on TFChain.
When `BurnTransactionReady` or `RefundTransactionReady` fires, the first validator to receive it
fetches all stored signatures from TFChain and builds a multi-sig Stellar transaction meeting
the threshold.

---

## Fee Mechanics

TFT uses 7 decimal places: `1 TFT = 10,000,000 base units`.
Default fees are 1 TFT each (configurable via `DEPOSIT_FEE` and `WITHDRAW_FEE`).

### Deposit (Stellar → TFChain)

Two enforcement layers:

1. **Bridge** (`mint.go`): if incoming Stellar amount ≤ `DepositFee`, bridge refunds on Stellar
   without proposing a mint (avoids an on-chain tx that would fail anyway).
2. **Pallet** (`execute_mint_transaction`): deducts `DepositFee` and mints `amount - deposit_fee`
   to the user's TFChain account.

### Withdraw (TFChain → Stellar)

One enforcement layer:

1. **Pallet** (`swap_to_stellar`): rejects if `amount ≤ WithdrawFee` with
   `AmountIsLessThanWithdrawFee`. If valid, stores `burn_amount = amount - withdraw_fee` in the
   event. The bridge reads this value directly and sends it to Stellar — no additional fee applied.

---

## Bridge Funding: Why `path_payment_strict_send`

The bridge Stellar deposit monitor only watches `payment` operations on the bridge account.
A `path_payment_strict_send` operation is structurally different and is not picked up by the
monitor — so funding the bridge wallet this way does not trigger a spurious refund on startup
rescan. Always use `path_payment_strict_send` when issuing TFT to the bridge account in test
setup.

---

## Idempotency and Crash Recovery

The bridge writes an idempotency record (bbolt DB at `<persistency>.idem.db`) before submitting
any Stellar transaction:

- `PROCESSING`: Stellar tx may or may not have been submitted
- `COMPLETED`: Stellar tx submitted and TFChain confirmation done

On restart, `reconcilePendingTransactions` scans all `PROCESSING` entries. For each:

1. Fetch recent outgoing Stellar transactions from Horizon (single request, reused for all checks)
2. Search by memo text (primary) then by sequence number (fallback)
3. If found: proceed directly to TFChain confirmation — no double-spend
4. If not found: log `"no Stellar tx found by memo or sequence, safe to retry"` — re-submit on
   next Ready event

---

## Known Limitations

1. **Sequence coordination under load**: All validators sign Stellar transactions at proposal time
   with a fixed sequence number. If another Stellar transaction from the bridge account is
   submitted between proposal and `Ready` events, the stored signatures reference a stale
   sequence → all validators get `tx_bad_seq` on first attempt. Recovery is automatic via the
   expiry cycle (~20 blocks, ~2 min delay). This is a pre-existing design constraint, not
   introduced by this PR.

2. **Stellar testnet Friendbot rate limits**: If Friendbot rejects account funding (rate limited
   or account already exists with funds), re-running `make bridge-accounts` generates fresh
   keypairs. This requires re-running `make bridge-setup` and `make bridge-start` as well —
   or simply re-run `make bridge-dev` for a clean slate.

3. **`--tmp` chain**: State is lost on TFChain restart. For persistent sessions, replace
   `--tmp` with `--base-path /tmp/tfchain-data` in the `bridge-tfchain-start` Makefile target.
