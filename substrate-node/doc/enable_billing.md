# Enable billing on tfchain

Every validator can now actively participate in billing of contracts. This by running the offchain worker with the `smct` key.

## How to enable on a validator

Run the validator with offchain worker enabled (by default it will be enabled). Generate a new key and put some funds on it.
Insert the funded key in the keystore:

```
tfchain key insert --keystore-path=/path-to-keystore --key-type "smct" --suri "words" --scheme=sr25519
```

The validator should now actively participate in billing.
