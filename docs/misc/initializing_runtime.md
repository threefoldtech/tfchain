# Initialize runtime for Threefold Grid 3.0

We need to initialize the runtime with a couple of objects to ensure the correct workflow. These policies can only be inserted with a `sudo` key, in order to ensure the objects cannot be changed by malicious users.

Objects should be inserted through the Polkadot UI for now.

## Pricing Policy

See this document: https://wiki.threefold.io/#/threefold__grid_pricing

We should insert the values for the prices defined in the wiki in our runtime.

Browse to `polkadot UI -> extrinsics -> sudo -> sudo(call) -> tfgridModule -> createPricingPolicy`

Given the values on the wiki the following object should be stored:

```json
{
    "name": "threefold_grid_v3_pricing",
    "su": {
        "value": 300000,
        "unit": Gigabytes
    },
    "cu": {
        "value": 600000,
        "unit": "Gigabytes"
    },
    "nu": {
        "value": 2000,
        "unit": "Gigabytes"
    },
    "cu": {
        "value": 100000,
        "unit": "Gigabytes"
    },
    "unique_name": {
        "value": 20000,
        "unit": "Gigabytes"
    },
    "domain_name": {
        "value": 40000,
        "unit": "Gigabytes"
    },
    "foundation_account": "foundation_address_here",
    "certified_sales_account": "certified_sales_account_here"
}
```

## Farming Policies

See this document: https://wiki.threefold.io/#/threefold__grid_pricing

We should insert the values for the prices defined in the wiki in our runtime.

Browse to `polkadot UI -> extrinsics -> sudo -> sudo(call) -> tfgridModule -> createFarmingPolicy`

Given the values on the wiki the following *objects* should be stored:


```json
{
    "name": "farming_policy_diy",
    "cu": 160000000,
    "su": 100000000,
    "nu": 2000000,
    "ipv4": 800000,
    "certification_type": Diy
}
```


```json
{
    "name": "farming_policy_certified",
    "cu": 200000000,
    "su": 120000000,
    "nu": 3000000,
    "ipv4": 1000000,
    "certification_type": Certified
}
```