# Substrate funding service

This service will fund accounts that require some funds to be able to submit extrinsics on our substrate based chain. This service will control a wallet that can activate user's accounts.

A user can only be activated if he/she passes the KYC that we provide. The KYC service will return the user's data signed. The signature and data and substrate account ID must be provided to this funding service in order to get the account activated. This service will validate the signature and data with the public key of the KYC service. If everything is valid, the user will receive 1 Substrate based tokens which he can use to operate on the substrate network. This is a one time operation only, the user cannot get these 1 tokens a second time unless his balance drops below 1.

## Endpoint

`/activate` will be the main endpoint for this service.

The body of the POST request should look like:

json
```
{
    "kycSignature": "sig",
    "data": {
        "name": "someName",
        "someOtherField": "someOtherValue",
        ...
    },
    "accountID": "substrateAccountID(ed25519)"
}
```

## Networks

We will run an activation service for each TF Grid network (mainnet, testnet, devnet).