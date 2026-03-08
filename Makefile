# ─────────────────────────────────────────────────────────────────────────────
# Bridge local development environment
# ─────────────────────────────────────────────────────────────────────────────
#
# Quick start (first run builds TFChain — takes 20-40 min):
#   make bridge-dev
#
# Subsequent runs (TFChain already built):
#   make bridge-dev
#
# Run tests against an already-running environment:
#   make bridge-test
#
# Configurable via environment variables:
#   TFCHAIN_URL        WebSocket URL for TFChain node  (default: ws://localhost:9944)
#   BRIDGE_TFT_FLOAT   TFT to mint into bridge wallet  (default: 20000)
#   USER_TFT_AMOUNT    TFT to mint into test user wallet (default: 1000)
#   DEPOSIT_FEE        Deposit fee in base units        (default: 10000000 = 1 TFT)
#   WITHDRAW_FEE       Withdraw fee in base units        (default: 10000000 = 1 TFT)
#   BRIDGE_ENV_FILE    Env file path                    (default: /tmp/bridge_local_env.sh)

# Paths (relative to repo root)
BRIDGE_DIR       := bridge/tfchain_bridge
BRIDGE_BIN       := $(BRIDGE_DIR)/tfchain_bridge_local
TFCHAIN_BIN      := substrate-node/target/release/tfchain
SCRIPTS_DIR      := scripts
BRIDGE_LOG       := /tmp/bridge_local.log
BRIDGE_PID_FILE  := /tmp/bridge_local.pid
TFCHAIN_LOG      := /tmp/tfchain_local.log
BRIDGE_ENV_FILE  ?= /tmp/bridge_local_env.sh
TFCHAIN_URL      ?= ws://localhost:9944

.PHONY: bridge-build bridge-build-tfchain bridge-accounts bridge-tfchain-start \
        bridge-tfchain-stop bridge-setup bridge-start bridge-stop bridge-test \
        bridge-dev bridge-clean bridge-help

## bridge-help: Show bridge dev environment targets
bridge-help:
	@grep -E '^## bridge-' $(MAKEFILE_LIST) | sed 's/## /  make /'

## bridge-build: Build the bridge binary (Go, fast ~5s)
bridge-build:
	@echo "==> Building bridge..."
	cd $(BRIDGE_DIR) && go build -o tfchain_bridge_local .
	@echo "==> Bridge binary: $(BRIDGE_BIN)"

## bridge-build-tfchain: Build TFChain node (Rust, slow first-time ~30min)
bridge-build-tfchain:
	@echo "==> Building TFChain node (this may take 20-40 minutes on first run)..."
	cd substrate-node && cargo build --release
	@echo "==> TFChain binary: $(TFCHAIN_BIN)"

## bridge-accounts: Generate Stellar accounts and write env file
bridge-accounts:
	@echo "==> Installing npm dependencies..."
	cd $(SCRIPTS_DIR) && npm install --silent
	@echo "==> Generating Stellar accounts..."
	BRIDGE_ENV_FILE=$(BRIDGE_ENV_FILE) node $(SCRIPTS_DIR)/bridge_accounts.js

## bridge-tfchain-start: Start TFChain dev node and wait until ready
bridge-tfchain-start:
	@test -f $(TFCHAIN_BIN) || (echo "ERROR: TFChain binary not found at $(TFCHAIN_BIN). Run: make bridge-build-tfchain" && exit 1)
	@echo "==> Starting TFChain dev node..."
	@pkill -f "$(notdir $(TFCHAIN_BIN)) --dev" 2>/dev/null || true
	@sleep 1
	nohup $(TFCHAIN_BIN) --dev --tmp > $(TFCHAIN_LOG) 2>&1 &
	@echo "==> Waiting for TFChain to be ready..."
	TFCHAIN_URL=$(TFCHAIN_URL) node $(SCRIPTS_DIR)/wait_for_node.js

## bridge-tfchain-stop: Stop the TFChain dev node
bridge-tfchain-stop:
	@pkill -f "$(notdir $(TFCHAIN_BIN)) --dev" 2>/dev/null && echo "==> TFChain stopped" || echo "==> TFChain was not running"

## bridge-setup: Configure TFChain bridge pallet (validators, fees, wallet address)
bridge-setup:
	@test -f $(BRIDGE_ENV_FILE) || (echo "ERROR: $(BRIDGE_ENV_FILE) not found. Run: make bridge-accounts" && exit 1)
	@echo "==> Configuring TFChain bridge pallet..."
	TFCHAIN_URL=$(TFCHAIN_URL) BRIDGE_ENV_FILE=$(BRIDGE_ENV_FILE) node $(SCRIPTS_DIR)/bridge_setup.js

## bridge-start: Start the bridge daemon
bridge-start:
	@test -f $(BRIDGE_BIN) || (echo "ERROR: Bridge binary not found. Run: make bridge-build" && exit 1)
	@test -f $(BRIDGE_ENV_FILE) || (echo "ERROR: $(BRIDGE_ENV_FILE) not found. Run: make bridge-accounts" && exit 1)
	@pkill -f "$(notdir $(BRIDGE_BIN))" 2>/dev/null || true
	@sleep 1
	@. $(BRIDGE_ENV_FILE) && \
	  nohup $(BRIDGE_BIN) \
	    --secret "$$BRIDGE_SECRET" \
	    --tfchainurl $(TFCHAIN_URL) \
	    --tfchainseed "//Alice" \
	    --bridgewallet "$$BRIDGE_ADDRESS" \
	    --persistency $(BRIDGE_DIR)/signer_local.json \
	    --network testnet \
	  > $(BRIDGE_LOG) 2>&1 & echo $$! > $(BRIDGE_PID_FILE)
	@echo "==> Bridge started (PID $$(cat $(BRIDGE_PID_FILE))), log: $(BRIDGE_LOG)"
	@echo "==> Waiting for bridge to be ready..."
	@timeout 30 sh -c 'until grep -q "bridge_started" $(BRIDGE_LOG) 2>/dev/null; do sleep 1; done' \
	  && echo "==> Bridge ready." || echo "==> Warning: bridge_started not seen in 30s, check $(BRIDGE_LOG)"

## bridge-stop: Stop the bridge daemon
bridge-stop:
	@if [ -f $(BRIDGE_PID_FILE) ]; then \
	  kill $$(cat $(BRIDGE_PID_FILE)) 2>/dev/null && echo "==> Bridge stopped" || true; \
	  rm -f $(BRIDGE_PID_FILE); \
	else \
	  pkill -f "$(notdir $(BRIDGE_BIN))" 2>/dev/null && echo "==> Bridge stopped" || echo "==> Bridge was not running"; \
	fi

## bridge-test: Run the E2E test suite against a running environment
bridge-test:
	@test -f $(BRIDGE_ENV_FILE) || (echo "ERROR: $(BRIDGE_ENV_FILE) not found. Run: make bridge-accounts" && exit 1)
	@echo "==> Running bridge E2E tests..."
	TFCHAIN_URL=$(TFCHAIN_URL) \
	BRIDGE_ENV_FILE=$(BRIDGE_ENV_FILE) \
	BRIDGE_PID_FILE=$(BRIDGE_PID_FILE) \
	BRIDGE_LOG_FILE=$(BRIDGE_LOG) \
	BRIDGE_BIN=$(BRIDGE_BIN) \
	node $(SCRIPTS_DIR)/bridge_tests.js

## bridge-clean: Stop everything and delete all local state
bridge-clean: bridge-stop bridge-tfchain-stop
	@echo "==> Cleaning local bridge state..."
	rm -f $(BRIDGE_DIR)/signer_local.json
	rm -f $(BRIDGE_DIR)/signer_local.json.idem.db
	rm -f $(BRIDGE_LOG) $(TFCHAIN_LOG) $(BRIDGE_PID_FILE)
	@echo "==> Clean done."

## bridge-dev: Full one-shot local dev environment (build → accounts → start → test)
## Note: TFChain is built only if binary is missing (slow first run, fast after).
bridge-dev: bridge-clean bridge-build $(TFCHAIN_BIN) bridge-accounts \
            bridge-tfchain-start bridge-setup bridge-start bridge-test

# ── Multi-validator targets ───────────────────────────────────────────────────

BRIDGE_MV_ENV_FILE ?= /tmp/bridge_mv_env.sh

.PHONY: bridge-mv-accounts bridge-mv-setup bridge-mv-start bridge-mv-stop \
        bridge-mv-test bridge-mv-clean bridge-mv-dev

## bridge-mv-accounts: Generate 3-validator Stellar accounts + 2-of-3 multi-sig
bridge-mv-accounts:
	@echo "==> Installing npm dependencies..."
	cd $(SCRIPTS_DIR) && npm install --silent
	@echo "==> Generating multi-validator Stellar accounts..."
	BRIDGE_MV_ENV_FILE=$(BRIDGE_MV_ENV_FILE) node $(SCRIPTS_DIR)/bridge_mv_accounts.js

## bridge-mv-setup: Configure TFChain for 3 validators (Alice, Bob, Charlie)
bridge-mv-setup:
	@test -f $(BRIDGE_MV_ENV_FILE) || (echo "ERROR: $(BRIDGE_MV_ENV_FILE) not found. Run: make bridge-mv-accounts" && exit 1)
	@echo "==> Configuring TFChain for multi-validator bridge..."
	TFCHAIN_URL=$(TFCHAIN_URL) BRIDGE_MV_ENV_FILE=$(BRIDGE_MV_ENV_FILE) \
	  node $(SCRIPTS_DIR)/bridge_mv_setup.js

## bridge-mv-start: Start 3 bridge daemons (Val1=Alice, Val2=Bob, Val3=Charlie)
bridge-mv-start:
	@test -f $(BRIDGE_BIN) || (echo "ERROR: Bridge binary not found. Run: make bridge-build" && exit 1)
	@test -f $(BRIDGE_MV_ENV_FILE) || (echo "ERROR: $(BRIDGE_MV_ENV_FILE) not found. Run: make bridge-mv-accounts" && exit 1)
	@pkill -f "$(notdir $(BRIDGE_BIN))" 2>/dev/null || true
	@sleep 1
	@. $(BRIDGE_MV_ENV_FILE) && for i in 1 2 3; do \
	  secret_var="VAL$${i}_STELLAR_SECRET"; \
	  seed_var="VAL$${i}_TFCHAIN_SEED"; \
	  secret=$$(eval echo \$$$${secret_var}); \
	  seed=$$([ $$i -eq 1 ] && echo "//Alice" || [ $$i -eq 2 ] && echo "//Bob" || echo "//Charlie"); \
	  nohup $(BRIDGE_BIN) \
	    --secret "$$secret" \
	    --tfchainurl $(TFCHAIN_URL) \
	    --tfchainseed "$$seed" \
	    --bridgewallet "$$BRIDGE_ADDRESS" \
	    --persistency $(BRIDGE_DIR)/signer_mv_$$i.json \
	    --network testnet \
	  > /tmp/bridge_mv_$$i.log 2>&1 & echo $$! > /tmp/bridge_mv_$$i.pid; \
	  echo "==> Val$$i started (PID $$(cat /tmp/bridge_mv_$$i.pid))"; \
	done
	@echo "==> Waiting for all 3 validators to be ready..."
	@for i in 1 2 3; do \
	  timeout 30 sh -c "until grep -q bridge_started /tmp/bridge_mv_$$i.log 2>/dev/null; do sleep 1; done" \
	    && echo "==> Val$$i ready" || echo "==> Warning: Val$$i bridge_started not seen"; \
	done

## bridge-mv-stop: Stop all 3 bridge daemons
bridge-mv-stop:
	@for i in 1 2 3; do \
	  if [ -f /tmp/bridge_mv_$$i.pid ]; then \
	    kill $$(cat /tmp/bridge_mv_$$i.pid) 2>/dev/null || true; \
	    rm -f /tmp/bridge_mv_$$i.pid; \
	    echo "==> Val$$i stopped"; \
	  fi; \
	done
	@pkill -f "$(notdir $(BRIDGE_BIN))" 2>/dev/null || true

## bridge-mv-test: Run multi-validator E2E test suite
bridge-mv-test:
	@test -f $(BRIDGE_MV_ENV_FILE) || (echo "ERROR: $(BRIDGE_MV_ENV_FILE) not found. Run: make bridge-mv-accounts" && exit 1)
	@echo "==> Running multi-validator E2E tests..."
	TFCHAIN_URL=$(TFCHAIN_URL) \
	BRIDGE_MV_ENV_FILE=$(BRIDGE_MV_ENV_FILE) \
	BRIDGE_BIN=$(BRIDGE_BIN) \
	BRIDGE_DIR=$(BRIDGE_DIR) \
	node $(SCRIPTS_DIR)/bridge_mv_tests.js

## bridge-mv-clean: Stop MV validators and delete MV state files
bridge-mv-clean: bridge-mv-stop
	@echo "==> Cleaning multi-validator bridge state..."
	rm -f $(BRIDGE_DIR)/signer_mv_*.json
	rm -f $(BRIDGE_DIR)/signer_mv_*.json.idem.db
	rm -f /tmp/bridge_mv_*.log /tmp/bridge_mv_*.pid
	@echo "==> MV clean done."

## bridge-mv-dev: Full one-shot multi-validator dev environment
bridge-mv-dev: bridge-mv-clean bridge-build $(TFCHAIN_BIN) bridge-mv-accounts \
               bridge-tfchain-start bridge-mv-setup bridge-mv-start bridge-mv-test

# Build TFChain only if binary doesn't exist (expensive Rust build)
$(TFCHAIN_BIN):
	@$(MAKE) bridge-build-tfchain

# ─────────────────────────────────────────────────────────────────────────────
# End bridge local development environment
# ─────────────────────────────────────────────────────────────────────────────

.PHONY: version-bump

# * Usage Examples:*
# type=patch make version-bump
# type=minor make version-bump
# type=major make version-bump
# ** skip increment spec_version in substrate-node/runtime/src/lib.rs **
# type=patch retain_spec_version=1 make version-bump
version-bump:
	set -e; \
	if [ "$(type)" = "patch" ] || [ "$(type)" = "minor" ] || [ "$(type)" = "major" ]; then \
		default_branch=$$(git symbolic-ref refs/remotes/origin/HEAD | sed 's@^refs/remotes/origin/@@'); \
		git checkout $$default_branch; \
		git pull origin $$default_branch; \
		new_version=$$(npx semver -i $(type) $$(jq -r .version clients/tfchain-client-js/package.json)); \
		branch_name="$$default_branch-bump-version-to-$$new_version"; \
		git checkout -b $$branch_name; \
		current_spec_version=$$(sed -n -e 's/^.*spec_version: \([0-9]\+\),$$/\1/p' substrate-node/runtime/src/lib.rs); \
		if [ -z "$${retain_spec_version}" ]; then \
			current_spec_version=$$(sed -n -e 's/^.*spec_version: \([0-9]\+\),$$/\1/p' substrate-node/runtime/src/lib.rs); \
			echo "Current spec_version: $$current_spec_version"; \
			new_spec_version=$$((current_spec_version + 1)); \
			echo "New spec_version: $$new_spec_version"; \
			sed -i "s/spec_version: $$current_spec_version,/spec_version: $$new_spec_version,/" substrate-node/runtime/src/lib.rs; \
		fi; \
		jq ".version = \"$$new_version\"" activation-service/package.json > temp.json && mv temp.json activation-service/package.json; \
		jq ".version = \"$$new_version\"" clients/tfchain-client-js/package.json > temp.json && mv temp.json clients/tfchain-client-js/package.json; \
		jq ".version = \"$$new_version\"" scripts/package.json > temp.json && mv temp.json scripts/package.json; \
		jq ".version = \"$$new_version\"" tools/fork-off-substrate/package.json > temp.json && mv temp.json tools/fork-off-substrate/package.json; \
		sed -i "s/^version = .*/version = \"$$new_version\"/" substrate-node/Cargo.toml; \
		sed -i "s/^version: .*/version: $$new_version/" substrate-node/charts/substrate-node/Chart.yaml; \
		sed -i "s/^appVersion: .*/appVersion: '$$new_version'/" substrate-node/charts/substrate-node/Chart.yaml; \
		sed -i "s/^version: .*/version: $$new_version/" bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml; \
		sed -i "s/^appVersion: .*/appVersion: '$$new_version'/" bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml; \
		sed -i "s/^version: .*/version: $$new_version/" activation-service/helm/tfchainactivationservice/Chart.yaml; \
		sed -i "s/^appVersion: .*/appVersion: '$$new_version'/" activation-service/helm/tfchainactivationservice/Chart.yaml; \
		cd substrate-node && cargo metadata -q 1> /dev/null && cd ..; \
		git add substrate-node/Cargo.toml substrate-node/Cargo.lock substrate-node/charts/substrate-node/Chart.yaml bridge/tfchain_bridge/chart/tfchainbridge/Chart.yaml activation-service/helm/tfchainactivationservice/Chart.yaml activation-service/package.json clients/tfchain-client-js/package.json scripts/package.json tools/fork-off-substrate/package.json substrate-node/runtime/src/lib.rs; \
		if [ -z "$${new_spec_version}" ]; then \
			git commit -m "Bump version to $$new_version"; \
		else \
			git commit -m "Bump version to $$new_version (spec v$$new_spec_version)"; \
		fi \
	else \
		echo "Invalid version type. Please use patch, minor, or major."; \
	fi
