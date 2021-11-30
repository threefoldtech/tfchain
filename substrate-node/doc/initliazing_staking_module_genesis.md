# Initializing staking module genesis config

Description for modifying the chainspec staking module genesis configuration.

This document describes an example of 2 initial validators.

**Every Validator need a Controller account and Stash account.**

## Generate keypairs for Validator 1

### Contoller account

This keypair will be the controller account for validator 1

```sh
subkey generate

Secret phrase `wire eagle crawl boss legal broken hockey mother topic board alley suit` is account:
  Secret seed:      0xe28f46eef6b3da9f366771b8f0df3fad9ce3e27ac432b6493d1148180e0d9a5e
  Public key (hex): 0x38232f469d8a557e518002bf7d1453626f227ef089bfe65d68f7bf790385c01a
  Account ID:       0x38232f469d8a557e518002bf7d1453626f227ef089bfe65d68f7bf790385c01a
  SS58 Address:     5DLJzR5pxFcufkk8Q1ZAtbbDK2egbkPWTsiwZy23U3aHzCpP
```

Inspect the ED25519 keypair

```sh
subkey inspect "wire eagle crawl boss legal broken hockey mother topic board alley suit" --scheme ed25519

Secret phrase `wire eagle crawl boss legal broken hockey mother topic board alley suit` is account:
  Secret seed:      0xe28f46eef6b3da9f366771b8f0df3fad9ce3e27ac432b6493d1148180e0d9a5e
  Public key (hex): 0xe5202b3670a3c71469131b6792b643e1a64cd8fc84fd4c7ff1b837236d008abb
  Account ID:       0xe5202b3670a3c71469131b6792b643e1a64cd8fc84fd4c7ff1b837236d008abb
  SS58 Address:     5HF8Nvq6uFbXtHaf36NpcXqabCeHYFSd3NmzxJt9YciXPY3N
```

### Stash account

This keypair will be the stash account for validator 1

```sh
subkey generate

Secret phrase ` remain ancient slam case text adapt unable wheat warm cheese walnut endless` is account:
  Secret seed:      0x182cdaa98fa1820380248119b3a754b51f717e68a13880e471dd7ae1d0337eae
  Public key (hex): 0x9ef2f192c767c48ab111eb41cf41b1eed5892926a9dbe04b45143d4f1210b471
  Account ID:       0x9ef2f192c767c48ab111eb41cf41b1eed5892926a9dbe04b45143d4f1210b471
  SS58 Address:     5Ff7bJgotSSR9EvDm1BsvVjS62V2t9yPSaWKBHmV8nXoZ3Y1
```

## Generate keypairs for Validator 2

### Contoller account

This keypair will be the controller account for validator 2

```sh
subkey generate

Secret phrase ` lazy toast trumpet zone pig april want goddess hollow grocery security buzz` is account:
  Secret seed:      0x00a22139f9976e894b6a54928be19f4b15e98b72a57517d4cc7a3e31d776fdf2
  Public key (hex): 0xc095eae7d37787125e9b031a801079db092384144efa0dbd632736b20b283644
  Account ID:       0xc095eae7d37787125e9b031a801079db092384144efa0dbd632736b20b283644
  SS58 Address:     5GRDaw4e3VA2iYp1sJbR8KXutQjSHZNAfJkvxFi39pKsYJUp
```

Inspect the ED25519 keypair

```sh
subkey inspect "lazy toast trumpet zone pig april want goddess hollow grocery security buzz" --scheme ed25519

Secret phrase ` lazy toast trumpet zone pig april want goddess hollow grocery security buzz` is account:
  Secret seed:      0x00a22139f9976e894b6a54928be19f4b15e98b72a57517d4cc7a3e31d776fdf2
  Public key (hex): 0x8c8e2c53519b283f2fd46b9b7e74eaff5f29b969db6bfae768ccdf1f0ceb359f
  Account ID:       0x8c8e2c53519b283f2fd46b9b7e74eaff5f29b969db6bfae768ccdf1f0ceb359f
  SS58 Address:     5FEzo5XTTLay7kNerxATivTrV1VyowdT3UBD8xoqDEywFwWc
```

### Stash account

```sh
subkey generate

Secret phrase `yellow globe swear always choose festival lounge mercy catalog normal base profit` is account:
  Secret seed:      0x37de876fe49391a6cb42fedd84937a2c2d3925e033c862f48a6c2993fb94dab9
  Public key (hex): 0xfabc4639c9052bae8ab1250935a08091e0a67421c534ec05b5e6f63a83513472
  Account ID:       0xfabc4639c9052bae8ab1250935a08091e0a67421c534ec05b5e6f63a83513472
  SS58 Address:     5HjTktTNow76ycutDDXmbrhThe3b1Zu6wPAB73E8zLKH9iFG
```

## Generate a chainspec

```sh
cargo build
./target/debug/tfchain build-spec --disable-default-bootnode --chain local > customSpec.json
```

Open `customSpec.json` and modify the following:

### Add Controller and Stash account of Validator 1 & 2 to the balances module

This will yield in 4 account having some initial balance.

```json
"palletBalances": {
    "balances": [
        [
            "5DLJzR5pxFcufkk8Q1ZAtbbDK2egbkPWTsiwZy23U3aHzCpP",
            100000
        ],
        [
            "5GRDaw4e3VA2iYp1sJbR8KXutQjSHZNAfJkvxFi39pKsYJUp",
            100000
        ],
        [
            "5Ff7bJgotSSR9EvDm1BsvVjS62V2t9yPSaWKBHmV8nXoZ3Y1",
            100000
        ],
        [
            "5HjTktTNow76ycutDDXmbrhThe3b1Zu6wPAB73E8zLKH9iFG",
            100000
        ]
    ]
},
```

### Add Stash account of Validator 1 & 2 to the `palletStaking` -> `invulnerables`

```json
"palletStaking:" {
    ...,
    "invulnerables": [
        "5Ff7bJgotSSR9EvDm1BsvVjS62V2t9yPSaWKBHmV8nXoZ3Y1",
        "5HjTktTNow76ycutDDXmbrhThe3b1Zu6wPAB73E8zLKH9iFG"
    ],
    ...
}
```

### Configure initial stakers in `palletStaking`

Every element in `stakers` array is a combination of `[stashAccount, controllerAccount, initialStakingBalance, StakingRole]`

```json
"palletStaking:" {
    ...,
    "stakers": [
        [
            "5Ff7bJgotSSR9EvDm1BsvVjS62V2t9yPSaWKBHmV8nXoZ3Y1",
            "5DLJzR5pxFcufkk8Q1ZAtbbDK2egbkPWTsiwZy23U3aHzCpP",
            100000000000,
            "Validator"
        ],
        [
            "5HjTktTNow76ycutDDXmbrhThe3b1Zu6wPAB73E8zLKH9iFG",
            "5GRDaw4e3VA2iYp1sJbR8KXutQjSHZNAfJkvxFi39pKsYJUp",
            100000000000,
            "Validator"
        ]
    ],
    ...
}
```

### Configure StakingPoolAccount

Set the value to `5CNposRewardAccount11111111111111111111111111FSU` , this is the account where the payouts will be paid from

```json
"palletStaking:" {
    ...,
    "stakingPoolAccount": "5CNposRewardAccount11111111111111111111111111FSU"
}
```

### Configure PalletSession

Every element in the keys array is a combination of:

- stash account
- stash account
- object:

    ```json
    { 
        "babe": "sr25519 account id of validator controller keypair",
        "grandpa": "ed25519 account id of validator controller keypair",
        "im_online": "sr25519 account id of validator controller keypair",
        "authority_discovery": "sr25519 account id of validator controller keypair",
    }
    ```

```json
"palletSession": {
    "keys": [
        [
            "5Ff7bJgotSSR9EvDm1BsvVjS62V2t9yPSaWKBHmV8nXoZ3Y1",
            "5Ff7bJgotSSR9EvDm1BsvVjS62V2t9yPSaWKBHmV8nXoZ3Y1",
            {
                "babe": "5DLJzR5pxFcufkk8Q1ZAtbbDK2egbkPWTsiwZy23U3aHzCpP",
                "grandpa": "5HF8Nvq6uFbXtHaf36NpcXqabCeHYFSd3NmzxJt9YciXPY3N",
                "im_online": "5DLJzR5pxFcufkk8Q1ZAtbbDK2egbkPWTsiwZy23U3aHzCpP",
                "authority_discovery": "5DLJzR5pxFcufkk8Q1ZAtbbDK2egbkPWTsiwZy23U3aHzCpP"
            }
        ],
            [
            "5HjTktTNow76ycutDDXmbrhThe3b1Zu6wPAB73E8zLKH9iFG",
            "5HjTktTNow76ycutDDXmbrhThe3b1Zu6wPAB73E8zLKH9iFG",
            {
                "babe": "5GRDaw4e3VA2iYp1sJbR8KXutQjSHZNAfJkvxFi39pKsYJUp",
                "grandpa": "5FEzo5XTTLay7kNerxATivTrV1VyowdT3UBD8xoqDEywFwWc",
                "im_online": "5GRDaw4e3VA2iYp1sJbR8KXutQjSHZNAfJkvxFi39pKsYJUp",
                "authority_discovery": "5GRDaw4e3VA2iYp1sJbR8KXutQjSHZNAfJkvxFi39pKsYJUp"
            }
        ]
    ]
},
```
