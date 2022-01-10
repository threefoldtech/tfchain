# Graphql

We have an instance deployed for our devnet on https://explorer.devnet.grid.tf/graphql/

## Query entities

A user can query entities as following:

json
```
{
    entities (limit:1){
        gridVersion,
        entityId,
        name,
        countryId,
        cityId,
        address
    }
}
```

## Query Twins

A user can query twins as following:

json
```
{
    twins (limit:1) {
        gridVersion,
        twinId,
        ip,
        address,
    }
}
```

## Query Farms

A user can query farms as following:

json
```
{
    farms (limit:1) {
        gridVersion,
        farmId,
        twinId,
        name,
        countryId,
        cityId,
        pricingPolicyId,
        certificationType,
        publicIPs{
            ip,
            gateway,
            contractId
        }
    }
}
```

## Query Nodes

A user can query nodes as following:

json
```
{
    nodes (limit:1) {
        gridVersion,
        nodeId,
        farmId,
        address,
        location {
            latitude,
            longitude
        },
        publicConfig {
            ipv4
            ipv6,
            gw4,
            gw6
        },
        hru,
        sru,
        cru,
        mru,
        role,
        twinId
    }
}
```

Query Cities 

A user can query all available cities in the system as following:

json
```
{
  cities(limit:1) {
    name,
    countryId
  }
}
```

Query Countries 

A user can query all available countries in the system as following:

json
```
{
  countries(limit:1) {
    name,
    code
  }
}
```