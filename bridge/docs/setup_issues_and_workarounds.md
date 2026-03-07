# Bridge Local Setup — Issues, Mistakes & Workarounds

This document tracks every mistake, unexpected issue, and its resolution encountered while setting up the local development environment and implementing fixes for [#1053](https://github.com/threefoldtech/tfchain/issues/1053) and [#1054](https://github.com/threefoldtech/tfchain/issues/1054).

---

## 1. Sudo Pallet Assumption (Planning Error)

**Phase:** Planning / Chain setup  
**Issue:** Initial setup script used `api.tx.sudo.sudo(...)` to call restricted functions like `addBridgeValidator`, `setFeeAccount`, `setWithdrawFee`, `setDepositFee`.  
**Root cause:** Assumed tfchain uses sudo pallet (common in dev Substrate chains). tfchain does NOT include sudo — it uses `EnsureRootOrCouncilApproval = EitherOfDiverse<EnsureRoot, pallet_collective::EnsureProportionAtLeast<3, 5>>`.  
**Workaround/Fix:** None needed — the genesis config for `--dev` chain already pre-seeds all bridge pallet configuration:
- Bridge validators (mnemonic "quarter between satisfy three sphere six soda boss cute decade old trend" + 2 others)
- Fee account (Alice in dev mode)
- Deposit fee: 10,000,000 muTFT (10 TFT)
- Withdraw fee: 10,000,000 muTFT (10 TFT)

So no restricted calls are needed for local setup at all. Only `userAcceptTc` + `createTwin` (regular signed calls) are required.

**Where confirmed:** `substrate-node/node/src/chain_spec.rs` → `testnet_genesis()` → `TFTBridgeModuleConfig` block.

---

## 2. `claude --print` Mode Doesn't Execute Code

**Phase:** Agent spawning  
**Issue:** First attempt used `claude --dangerously-skip-permissions --print "$(task)"` piped to `nohup`. The log file stayed empty for minutes despite the process running.  
**Root cause:** `--print` mode is a one-shot text generation mode — it outputs a response but does NOT execute shell commands or use tools. It's equivalent to asking Claude a question in text mode.  
**Workaround:** Use interactive TUI mode (without `--print`) with `pty:true`. This is the mode where Claude Code actually runs bash commands, edits files, etc.

**Correct pattern:**
```bash
cd /project && claude --dangerously-skip-permissions "task description"
# pty:true, background:true, yieldMs:15000+
# Then accept bypass prompt: send-keys ["down", "enter"]
```

---

## 3. PTY Broken by Output Pipe

**Phase:** Agent spawning  
**Issue:** First PTY-mode attempt appended `| head -20` to the command. The agent session showed no output, bypass prompt couldn't be accepted.  
**Root cause:** Piping stdout of an interactive PTY application (`| head -20`) breaks the terminal allocation. The PTY requires a direct terminal connection — piping severs it.  
**Workaround:** Never pipe the output of a PTY coding agent command. Monitor it via `process log` / `process poll` instead.

---

## 4. Claude Code Auto-Updated Mid-Session and Stalled

**Phase:** Implementation (after code writing completed)  
**Issue:** After successfully writing all code files and passing `go vet`, the agent output showed `Auto-updating…` and then went idle at the prompt. It did not continue to the build/test phase.  
**Root cause:** Claude Code triggered an automatic self-update (`✳ Claude Code` icon change). After updating, the session was left at the interactive prompt with no pending task.  
**Workaround:** Detect the stall (via `process poll` returning "still running" but no progress), then use `process send-keys` with `literal` to inject the next instruction into the running session:
```
process send-keys literal:"Continue from Phase 1: build the substrate node..."
process send-keys keys:["enter"]
```

---

## 5. Rust Not on PATH in Agent's Shell Environment

**Phase:** Rust build  
**Issue:** Agent ran `rustc --version` and got `command not found` (exit 127), even though Rust is installed via rustup.  
**Root cause:** The agent's shell environment doesn't source `~/.bashrc` or `~/.cargo/env` automatically. Rustup installs to `~/.cargo/bin` but this isn't in the default PATH for non-interactive shells.  
**Workaround:** Prefix all cargo/rustc commands with the explicit PATH:
```bash
export PATH="$HOME/.cargo/bin:$HOME/sdk/go/bin:$HOME/go/bin:$PATH"
cargo build ...
```

---

## 6. `cargo build` Output Tail Misleading

**Phase:** Rust build  
**Issue:** Running `cargo build 2>&1 | tail -20` in background appeared to complete instantly with exit code 0, but no binary existed.  
**Root cause:** The `tail -20` subprocess exited after receiving the first 20 lines of output, causing the pipe to close and `cargo build` to receive SIGPIPE. Cargo may have partially compiled but the binary wasn't produced.  
**Workaround:** Run cargo build without piping. Log full output to a file if needed:
```bash
cargo build > /tmp/cargo_build.log 2>&1
# or just: cargo build 2>&1  (let it stream to the PTY)
```

---

## 7. `openclaw cron add` Missing Required Flags

**Phase:** Scheduling the progress reminder  
**Issue:** Multiple iterations needed to get the cron command right:
- Missing `--name` flag → error
- Used `--prompt` (doesn't exist) → error
- Used `--exact` with `--every` (only valid with `--cron`) → error
- Used `--session main` without `--system-event` → error

**Workaround:** Correct flags for an isolated agent cron job with Telegram delivery:
```bash
openclaw cron add \
  --name "job-name" \
  --every "15m" \
  --session isolated \
  --message "task description" \
  --channel telegram \
  --announce
```

---

## 8. Pre-existing Bug Found During Implementation

**Phase:** Code review / implementation  
**Issue:** In `bridge/tfchain_bridge/pkg/bridge/bridge.go`, the event loop error handling used:
```go
return errors.Wrap(err, "failed to get tfchain events")
```
But `err` at that point is `nil` (from the outer scope). The actual error is `data.Err`.  
**Fix:** Changed to:
```go
return errors.Wrap(data.Err, "failed to get tfchain events")
```
**Impact:** Without this fix, tfchain subscription errors would be silently swallowed, making the bridge appear to run normally while it's actually not processing events.

---

## 9. `defer idempotency.Close()` Placement

**Phase:** Code review  
**Issue:** Initial implementation placed `defer idempotency.Close()` inside a function that could return early, leaving the bbolt DB open.  
**Fix:** Moved the defer to `NewBridge()` return path and added explicit close in the `Start()` shutdown path.

---

## 10. `ItemFailed` Index Mapping in Batch Result Parsing

**Phase:** Implementation — batch.go  
**Issue:** When parsing `Utility_ItemFailed` events from a batch extrinsic, the initial implementation used the event index directly to map back to the original call index. This is wrong — `ItemFailed.index` is the call index within the batch, not the event position.  
**Fix:** Used `event.ItemFailed.Index` (the field on the event struct) directly as the failed call index, which correctly maps to the original slice of calls.

---

## 11. Stellar Testnet TFT Faucet Has No Liquidity

**Phase:** Testnet account funding  
**Issue:** `stellar-utils faucet --secret <secret>` failed with a path payment error. The DEX path swap (XLM → TFT) found no matching orders.  
**Root cause:** Stellar testnet DEX has zero TFT liquidity. The official ThreeFold testnet TFT issuer (`GA47YZA3PKFUZMPLQ3B5F2E3CJIB57TGGU7SPCQT2WAEYKN766PWIMB3`) doesn't actively maintain testnet DEX orders, so path payment swaps always fail.  
**Workaround:** Create a custom TFT issuer on testnet:
1. Generate a new Stellar keypair — this becomes the issuer
2. Fund it via friendbot: `https://friendbot.stellar.org/?addr=<issuer_address>`
3. Add a trustline from each test account to the custom issuer
4. Send TFT directly from issuer to test accounts (no DEX needed)
5. Patch `TFTTest` constant in `stellar.go` to point to the custom issuer

**Custom testnet issuer used (test only, not for production):**
- Address: `GDPARZINMN52LJMVZSQPOEDHC2TWKJVFZSNHKDP4OUH6RI4PMXH4JA6Q`

> ⚠️ **Important:** Never use a custom issuer in production. Mainnet uses `GBOVQKJYHXRR3DX6NOX2RRYFRCUMSADGDESTDNBDS6CDVLGVESRTAC47`.

---

## 12. Stellar Memo Not Set on Withdraw Transactions (Bug Found During Testing)

**Phase:** Crash recovery testing  
**Issue:** Crash recovery relies on `FindPaymentByMemo` to check if a Stellar withdrawal was already submitted before the bridge crashed. During testing, the reconciliation consistently returned "no Stellar tx found" even when the Stellar tx WAS submitted.  
**Root cause:** `CreatePaymentWithSignaturesAndSubmit` submitted Stellar transactions without setting a memo field. `FindPaymentByMemo` searches for `memo_type=text` but found nothing, because withdrawals went out with no memo at all.

Additionally, when the memo was added only to the submission function and not the signing function (`CreatePaymentAndReturnSignature`), the signature verification failed — because the Stellar tx hash is computed over all transaction fields including memo. Validators signed a hash without the memo; submission with memo produced a different hash; signatures were invalid.

**Fix:** Added `txnBuild.Memo = txnbuild.MemoText(fmt.Sprint(txID))` to **both** `CreatePaymentAndReturnSignature` and `CreatePaymentWithSignaturesAndSubmit`. The memo must be set at signing time and submission time to maintain hash consistency across all validators.

**Verification:** After fix, all Stellar withdraw transactions have `memo_type=text` with the burn tx ID as value. Confirmed via Horizon API.

---
