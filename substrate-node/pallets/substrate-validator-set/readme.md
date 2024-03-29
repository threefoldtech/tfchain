# Substrate Validator Set

**Fork from https://github.com/gautamdhameja/substrate-validator-set commit hash: f83f1000c298c376ab180787eb73d0a0fdb740e0**

Code changes are in `diff.patch`

A [Substrate](https://github.com/paritytech/substrate/) pallet to add/remove validators using extrinsics, in Substrate-based PoA networks.

**Note: Current build is compatible with Substrate [v3.0.0](https://github.com/paritytech/substrate/releases/tag/v3.0.0) release.**

## Demo

To see this pallet in action in a Substrate runtime, watch this video - https://www.youtube.com/watch?v=lIYxE-tOAdw

## Setup with Substrate Node Template

*   Add the module's dependency in the `Cargo.toml` of your runtime directory. Make sure to enter the correct path or git url of the pallet as per your setup.

```toml
validatorset = { 
  version = "3.0.0", 
  package = "substrate-validator-set", 
  git = "https://github.com/threefoldtech/tfchain_pallets", 
  default-features = false
  branch = "development"
}
pallet-session = { default-features = false, version = '3.0.0' }
...
std = [
    ...
    'validatorset/std',
    'sp-session/std',
]
```

*   Make sure that you also have the Substrate [session pallet](https://github.com/paritytech/substrate/tree/master/frame/session) as part of your runtime. This is because the validator-set pallet is based on the session pallet.

*   Import `OpaqueKeys` in your `runtime/src/lib.rs`.

```rust
use sp_runtime::traits::{
	AccountIdLookup, BlakeTwo256, Block as BlockT, Verify, IdentifyAccount, NumberFor, OpaqueKeys
};
```

*   Declare the pallet in your `runtime/src/lib.rs`.

```rust
impl validatorset::Config for Runtime {
	type Event = Event;
}
```

*   Also, declare the session pallet in  your `runtime/src/lib.rs`. The type configuration of session pallet would depend on the ValidatorSet pallet as shown below.

```rust
impl pallet_session::Config for Runtime {
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type ShouldEndSession = ValidatorSet;
	type SessionManager = ValidatorSet;
	type Event = Event;
	type Keys = opaque::SessionKeys;
	type NextSessionRotation = ValidatorSet;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = validatorset::ValidatorOf<Self>;
	type DisabledValidatorsThreshold = ();
	type WeightInfo = ();
}
```

*   Add both `session` and `validatorset` pallets in `construct_runtime` macro. **Make sure to add them before `Aura` and `Grandpa` pallets and after `Balances`.**

```rust
construct_runtime!(
	pub enum Runtime
	{
		...
    	Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
		Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
		ValidatorSet: validatorset::{Module, Call, Storage, Event<T>, Config<T>},
		Aura: aura::{Module, Config<T>, Inherent(Timestamp)},
		Grandpa: grandpa::{Module, Call, Storage, Config, Event},
        ...
        ...
	}
);
```

*   Add genesis config in the `chain_spec.rs` file for `session` and `validatorset` pallets, and update it for `Aura` and `Grandpa` pallets. Because the validators are provided by the `session` pallet, we do not initialize them explicitly for `Aura` and `Grandpa` pallets. Order is important, notice that `pallet_session` is declared after `pallet_balances` since it depends on it (session accounts should have some balance).

```rust
fn testnet_genesis(initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool) -> GenesisConfig {
	GenesisConfig {
		...,
    pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		validatorset: Some(ValidatorSetConfig {
			validators: initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		}),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone()))
			}).collect::<Vec<_>>(),
		}),
		aura: Some(AuraConfig {
			authorities: vec![],
		}),
		grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
	}
}
```

*   Make sure you have the same number and order of session keys for your runtime. First in `runtime/src/lib.rs`:

```rust
pub struct SessionKeys {
	pub aura: Aura,
	pub grandpa: Grandpa,
}
```

*   And then in `src/chain_spec.rs`:

```rust
fn session_keys(
	aura: AuraId,
	grandpa: GrandpaId,
) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

pub fn authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AuraId,
	GrandpaId
) {
	(
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<AuraId>(seed),
		get_from_seed::<GrandpaId>(seed)
	)
}
```

*   `cargo build --release` and then `cargo run --release -- --dev`

## Sample

The usage of this pallet are demonstrated in the [Substrate permissioning sample](https://github.com/gautamdhameja/substrate-permissioning).

## Additional Types for Polkadot JS Apps/API

```json
{
  "Keys": "SessionKeys2"
}
```

## Disclaimer

This code not audited and reviewed for production use cases. You can expect bugs and security vulnerabilities. Do not use it as-is in real applications.
