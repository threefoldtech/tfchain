.PHONY: version-bump

# * Usage Examples:*
# type=patch make version-bump
# type=minor make version-bump
# type=major make version-bump
# ** skip increment spce_version in substrate-node/runtime/src/lib.rs **
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
		git commit -m "Bump version to $$new_version (spec v$$new_spec_version)"; \
	else \
		echo "Invalid version type. Please use patch, minor, or major."; \
	fi

