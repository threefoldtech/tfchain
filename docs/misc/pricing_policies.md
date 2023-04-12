# Pricing Policies

A Pricing Policy defines the pricing on chain for cloud units. See [library](https://library.threefold.me/info/threefold/#/tfgrid/grid/pricing). 

The structure on chain looks as following:

```
pub struct PricingPolicy<AccountId> {
    pub version: u32,
    pub id: u32,
    pub name: Vec<u8>,
    pub su: Policy,
    pub cu: Policy,
    pub nu: Policy,
    pub ipu: Policy,
    pub unique_name: Policy,
    pub domain_name: Policy,
    pub foundation_account: AccountId,
    pub certified_sales_account: AccountId,
}
pub struct Policy {
    pub value: u32,
    pub unit: Unit,
}
pub enum Unit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terrabytes,
}
```

- version: version of the policy
- id: identifier of the policy
- name: name identifying the policy
- su, cu, nu, ipu, unique_name, domain_name: is an object of type `Policy`. This object holds the value of the Pricing Policy (as defined in the library) in units usd. A `Policy` type also has a `Unit`
- foundation_account: Is an account defined by Threefold that get's a % of the rewards of cultivation (usage of capacity).
- certified_sales_account: Is an account defined by Threefold that get's a % of the rewards of cultivation (usage of capacity).

A pricing policy is statically assigned to a farmer. Whenever a farm is created, it's assigned pricing policy is the one with ID: 1. (This will probably need to change in the future)

## Policy Explained

The Policy struct has a value and a unit. The value is always expressed in unit usd. So for example if the library defines CU as 30.56 mUSD/hour then the value should be `305600`. On chain we always work with the lowest of unit this is why we need to format the price from mUSD to unit usd. 

The `Unit` type of the `Policy` is a Enumeration that holds information about how the pricing is defined, for example in the library, SU is defined in Terrabyte so we can also hold into account that it's expressed in Terrabytes. We need this because the 3Nodes send consumption of capacity in bytes and we need to calculate the amount due based on the price defined in Terrabytes in this example.

## Notes

For unqiue_name and domain_name a Policy types was not really needed, but we used the same type everywhere to have code consistency. We only really rely on this Policy type in certain places where we need it (smart contract billing).