SHELL := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

# ─────────────────────────────────────────────────────────────────────────────
# Bridge local development environment
# ─────────────────────────────────────────────────────────────────────────────
#
# Quick start (first run builds TFChain — takes 20-40 min):
#   make bridge-dev        # single validator
#   make bridge-mv-dev     # 3 validators, 2-of-3 multi-sig
#
# Configurable:
#   TFCHAIN_URL         WebSocket URL  (default: ws://localhost:9944)
#   BRIDGE_ENV_FILE     Env file path  (default: /tmp/bridge_local_env.sh)
#   BRIDGE_MV_ENV_FILE  MV env file    (default: /tmp/bridge_mv_env.sh)

BRIDGE_DIR        := bridge/tfchain_bridge
BRIDGE_BIN        := $(BRIDGE_DIR)/tfchain_bridge_local
TFCHAIN_BIN       := substrate-node/target/release/tfchain
SCRIPTS_DIR       := scripts

BRIDGE_LOG        := /tmp/bridge_local.log
BRIDGE_PID_FILE   := /tmp/bridge_local.pid
TFCHAIN_LOG       := /tmp/tfchain_local.log
TFCHAIN_PID_FILE  := /tmp/tfchain_local.pid

BRIDGE_ENV_FILE    ?= /tmp/bridge_local_env.sh
BRIDGE_MV_ENV_FILE ?= /tmp/bridge_mv_env.sh
TFCHAIN_URL        ?= ws://localhost:9944

.PHONY: bridge-help bridge-build bridge-build-tfchain \
        bridge-accounts bridge-mv-accounts \
        bridge-tfchain-start bridge-tfchain-stop \
        bridge-setup bridge-mv-setup \
        bridge-start bridge-stop bridge-test bridge-clean bridge-dev \
        bridge-mv-start bridge-mv-stop bridge-mv-test bridge-mv-clean bridge-mv-dev

# ─────────────────────────────────────────────────────────────────────────────
# Daemon helpers
# ─────────────────────────────────────────────────────────────────────────────

# Start a daemon, write its PID, verify it's alive after 1s.
# Usage: $(call start_daemon,Name,command,logfile,pidfile)
define start_daemon
	@echo "==> Starting $(1)..."
	@nohup $(2) > $(3) 2>&1 & echo $$! > $(4)
	@sleep 1
	@PID=$$(cat $(4)); \
	if kill -0 $$PID 2>/dev/null; then \
	  echo "==> $(1) started (PID $$PID)"; \
	else \
	  echo "ERROR: $(1) failed to start. Check $(3)"; \
	  exit 1; \
	fi
endef

# Like start_daemon but sources an env file first (in the same shell as nohup,
# so exported variables are inherited by the child process).
# Usage: $(call start_daemon_with_env,Name,envfile,command,logfile,pidfile)
define start_daemon_with_env
	@echo "==> Starting $(1)..."
	@. $(2) && nohup $(3) > $(4) 2>&1 & echo $$! > $(5)
	@sleep 1
	@PID=$$(cat $(5)); \
	if kill -0 $$PID 2>/dev/null; then \
	  echo "==> $(1) started (PID $$PID)"; \
	else \
	  echo "ERROR: $(1) failed to start. Check $(4)"; \
	  exit 1; \
	fi
endef

# Stop a daemon via its PID file. No pkill — avoids terminating the make shell.
# Usage: $(call stop_daemon,Name,pidfile)
define stop_daemon
	@if [ -f $(2) ]; then \
	  PID=$$(cat $(2)); \
	  if kill -0 $$PID 2>/dev/null; then \
	    kill $$PID; \
	    echo "==> $(1) stopped (PID $$PID)"; \
	  else \
	    echo "==> $(1) process not running (stale PID $$PID)"; \
	  fi; \
	  rm -f $(2); \
	else \
	  echo "==> $(1) not running (no PID file)"; \
	fi
endef

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────

bridge-help:
	@grep -E '^bridge-[a-z-]+:' $(MAKEFILE_LIST) | sed 's/:.*//' | sort

# ─────────────────────────────────────────────────────────────────────────────
# Build
# ─────────────────────────────────────────────────────────────────────────────

bridge-build:
	@echo "==> Building bridge..."
	cd $(BRIDGE_DIR) && go build -o tfchain_bridge_local .
	@echo "==> Bridge binary: $(BRIDGE_BIN)"

bridge-build-tfchain:
	@echo "==> Building TFChain (may take 20-40 min first time)..."
	cd substrate-node && cargo build --release
	@echo "==> TFChain binary: $(TFCHAIN_BIN)"

# ─────────────────────────────────────────────────────────────────────────────
# Accounts
# ─────────────────────────────────────────────────────────────────────────────

bridge-accounts:
	cd $(SCRIPTS_DIR) && npm install --silent
	BRIDGE_ENV_FILE=$(BRIDGE_ENV_FILE) node $(SCRIPTS_DIR)/bridge_accounts.js

bridge-mv-accounts:
	cd $(SCRIPTS_DIR) && npm install --silent
	BRIDGE_MV_ENV_FILE=$(BRIDGE_MV_ENV_FILE) node $(SCRIPTS_DIR)/bridge_mv_accounts.js

# ─────────────────────────────────────────────────────────────────────────────
# TFChain
# ─────────────────────────────────────────────────────────────────────────────

bridge-tfchain-start:
	@test -f $(TFCHAIN_BIN) || { echo "Run: make bridge-build-tfchain"; exit 1; }
	$(call stop_daemon,TFChain,$(TFCHAIN_PID_FILE))
	$(call start_daemon,TFChain,$(TFCHAIN_BIN) --dev --tmp,$(TFCHAIN_LOG),$(TFCHAIN_PID_FILE))
	@echo "==> Waiting for node..."
	TFCHAIN_URL=$(TFCHAIN_URL) node $(SCRIPTS_DIR)/wait_for_node.js

bridge-tfchain-stop:
	$(call stop_daemon,TFChain,$(TFCHAIN_PID_FILE))

# ─────────────────────────────────────────────────────────────────────────────
# Bridge setup
# ─────────────────────────────────────────────────────────────────────────────

bridge-setup:
	@test -f $(BRIDGE_ENV_FILE) || { echo "Run: make bridge-accounts"; exit 1; }
	TFCHAIN_URL=$(TFCHAIN_URL) \
	BRIDGE_ENV_FILE=$(BRIDGE_ENV_FILE) \
	node $(SCRIPTS_DIR)/bridge_setup.js

bridge-mv-setup:
	@test -f $(BRIDGE_MV_ENV_FILE) || { echo "Run: make bridge-mv-accounts"; exit 1; }
	TFCHAIN_URL=$(TFCHAIN_URL) \
	BRIDGE_MV_ENV_FILE=$(BRIDGE_MV_ENV_FILE) \
	node $(SCRIPTS_DIR)/bridge_mv_setup.js

# ─────────────────────────────────────────────────────────────────────────────
# Bridge daemon (single-validator)
# ─────────────────────────────────────────────────────────────────────────────
#
# Note: start_daemon_with_env sources BRIDGE_ENV_FILE inside the same shell
# as nohup so that BRIDGE_SECRET and BRIDGE_ADDRESS are inherited by the
# child process. Sourcing in a separate @-line would not work — each @-line
# is an independent shell invocation.

bridge-start:
	@test -f $(BRIDGE_BIN) || { echo "Run: make bridge-build"; exit 1; }
	@test -f $(BRIDGE_ENV_FILE) || { echo "Run: make bridge-accounts"; exit 1; }
	$(call start_daemon_with_env,Bridge,$(BRIDGE_ENV_FILE),$(BRIDGE_BIN) \
	  --secret "$$BRIDGE_SECRET" \
	  --tfchainurl $(TFCHAIN_URL) \
	  --tfchainseed "quarter between satisfy three sphere six soda boss cute decade old trend" \
	  --bridgewallet "$$BRIDGE_ADDRESS" \
	  --persistency $(BRIDGE_DIR)/signer_local.json \
	  --network testnet,$(BRIDGE_LOG),$(BRIDGE_PID_FILE))
	@echo "==> Waiting for bridge to be ready..."
	@i=0; \
	while [ $$i -lt 30 ] && ! grep -q "bridge_started" $(BRIDGE_LOG) 2>/dev/null; do \
	  sleep 1; i=$$((i+1)); \
	done; \
	if grep -q "bridge_started" $(BRIDGE_LOG) 2>/dev/null; then \
	  echo "==> Bridge ready."; \
	else \
	  echo "==> Warning: bridge_started not seen in 30s, check $(BRIDGE_LOG)"; \
	fi

bridge-stop:
	$(call stop_daemon,Bridge,$(BRIDGE_PID_FILE))

bridge-test:
	@test -f $(BRIDGE_ENV_FILE) || { echo "Run: make bridge-accounts"; exit 1; }
	@echo "==> Running bridge E2E tests..."
	TFCHAIN_URL=$(TFCHAIN_URL) \
	BRIDGE_ENV_FILE=$(BRIDGE_ENV_FILE) \
	BRIDGE_PID_FILE=$(BRIDGE_PID_FILE) \
	BRIDGE_LOG_FILE=$(BRIDGE_LOG) \
	BRIDGE_BIN=$(BRIDGE_BIN) \
	VAL1_TFCHAIN_SEED="quarter between satisfy three sphere six soda boss cute decade old trend" \
	node $(SCRIPTS_DIR)/bridge_tests.js

bridge-clean: bridge-stop bridge-tfchain-stop
	rm -f $(BRIDGE_DIR)/signer_local.json
	rm -f $(BRIDGE_DIR)/signer_local.json.idem.db
	rm -f $(BRIDGE_LOG) $(TFCHAIN_LOG)
	rm -f $(BRIDGE_PID_FILE) $(TFCHAIN_PID_FILE)

bridge-dev: bridge-clean bridge-build $(TFCHAIN_BIN) bridge-accounts \
            bridge-tfchain-start bridge-setup bridge-start bridge-test

# ─────────────────────────────────────────────────────────────────────────────
# Multi-validator bridge
# ─────────────────────────────────────────────────────────────────────────────
#
# Each validator is started in its own @-line so that $! captures the correct
# PID for each process. Validators source BRIDGE_MV_ENV_FILE in the same shell
# as nohup so env vars are inherited.

bridge-mv-start:
	@test -f $(BRIDGE_BIN) || { echo "Run: make bridge-build"; exit 1; }
	@test -f $(BRIDGE_MV_ENV_FILE) || { echo "Run: make bridge-mv-accounts"; exit 1; }
	@. $(BRIDGE_MV_ENV_FILE) && \
	  nohup $(BRIDGE_BIN) \
	    --secret "$$VAL1_STELLAR_SECRET" \
	    --tfchainurl $(TFCHAIN_URL) \
	    --tfchainseed "quarter between satisfy three sphere six soda boss cute decade old trend" \
	    --bridgewallet "$$BRIDGE_ADDRESS" \
	    --persistency $(BRIDGE_DIR)/signer_mv_1.json \
	    --network testnet \
	  > /tmp/bridge_mv_1.log 2>&1 & echo $$! > /tmp/bridge_mv_1.pid
	@echo "==> Val1 started (PID $$(cat /tmp/bridge_mv_1.pid))"
	@. $(BRIDGE_MV_ENV_FILE) && \
	  nohup $(BRIDGE_BIN) \
	    --secret "$$VAL2_STELLAR_SECRET" \
	    --tfchainurl $(TFCHAIN_URL) \
	    --tfchainseed "employ split promote annual couple elder remain cricket company fitness senior fiscal" \
	    --bridgewallet "$$BRIDGE_ADDRESS" \
	    --persistency $(BRIDGE_DIR)/signer_mv_2.json \
	    --network testnet \
	  > /tmp/bridge_mv_2.log 2>&1 & echo $$! > /tmp/bridge_mv_2.pid
	@echo "==> Val2 started (PID $$(cat /tmp/bridge_mv_2.pid))"
	@. $(BRIDGE_MV_ENV_FILE) && \
	  nohup $(BRIDGE_BIN) \
	    --secret "$$VAL3_STELLAR_SECRET" \
	    --tfchainurl $(TFCHAIN_URL) \
	    --tfchainseed "remind bird banner word spread volume card keep want faith insect mind" \
	    --bridgewallet "$$BRIDGE_ADDRESS" \
	    --persistency $(BRIDGE_DIR)/signer_mv_3.json \
	    --network testnet \
	  > /tmp/bridge_mv_3.log 2>&1 & echo $$! > /tmp/bridge_mv_3.pid
	@echo "==> Val3 started (PID $$(cat /tmp/bridge_mv_3.pid))"
	@echo "==> Waiting for validators to be ready..."
	@for i in 1 2 3; do \
	  j=0; \
	  while [ $$j -lt 30 ] && ! grep -q bridge_started /tmp/bridge_mv_$$i.log 2>/dev/null; do \
	    sleep 1; j=$$((j+1)); \
	  done; \
	  if grep -q bridge_started /tmp/bridge_mv_$$i.log 2>/dev/null; then \
	    echo "==> Val$$i ready"; \
	  else \
	    echo "==> WARNING: Val$$i bridge_started not seen in 30s. Last lines:"; \
	    tail -5 /tmp/bridge_mv_$$i.log 2>/dev/null || echo "(no log)"; \
	  fi; \
	done

bridge-mv-stop:
	@for i in 1 2 3; do \
	  PID_FILE=/tmp/bridge_mv_$$i.pid; \
	  if [ -f $$PID_FILE ]; then \
	    PID=$$(cat $$PID_FILE); \
	    if kill -0 $$PID 2>/dev/null; then \
	      kill $$PID; \
	      echo "==> Val$$i stopped (PID $$PID)"; \
	    else \
	      echo "==> Val$$i process not running (stale PID $$PID)"; \
	    fi; \
	    rm -f $$PID_FILE; \
	  else \
	    echo "==> Val$$i not running (no PID file)"; \
	  fi; \
	done

bridge-mv-test:
	@test -f $(BRIDGE_MV_ENV_FILE) || { echo "Run: make bridge-mv-accounts"; exit 1; }
	@echo "==> Running multi-validator E2E tests..."
	TFCHAIN_URL=$(TFCHAIN_URL) \
	BRIDGE_MV_ENV_FILE=$(BRIDGE_MV_ENV_FILE) \
	BRIDGE_BIN=$(BRIDGE_BIN) \
	BRIDGE_DIR=$(BRIDGE_DIR) \
	node $(SCRIPTS_DIR)/bridge_mv_tests.js

bridge-mv-clean: bridge-mv-stop
	rm -f $(BRIDGE_DIR)/signer_mv_*.json
	rm -f $(BRIDGE_DIR)/signer_mv_*.json.idem.db
	rm -f /tmp/bridge_mv_*.log
	rm -f /tmp/bridge_mv_*.pid

bridge-mv-dev: bridge-mv-clean bridge-build $(TFCHAIN_BIN) bridge-mv-accounts \
               bridge-tfchain-start bridge-mv-setup bridge-mv-start bridge-mv-test

# Build TFChain only if binary is missing (expensive Rust build)
$(TFCHAIN_BIN):
	$(MAKE) bridge-build-tfchain

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
		new_spec_version=""; \
		if [ -z "$${retain_spec_version:-}" ]; then \
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
		if [ -z "$${new_spec_version:-}" ]; then \
			git commit -m "Bump version to $$new_version"; \
		else \
			git commit -m "Bump version to $$new_version (spec v$$new_spec_version)"; \
		fi \
	else \
		echo "Invalid version type. Please use patch, minor, or major."; \
	fi
