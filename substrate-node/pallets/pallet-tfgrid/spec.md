# Threefold Grid Database

The puprose of the Grid Database on Substrate is to decentralise the way we work with identities.

Following identities are stored on Substrate:

- Entities (physical humain beings, only 1 entity per human).
- Twin (Digital Assistant).
- Farmers (Threefold grid Farmers).
- Node (Threefold grid physical Nodes).

A Twin object is digital representation of either a User, Node or a Farm. It facilitates a way to communicate against the chain.

[Entity Relationship Diagram](./diagram.md)

## Prelude

This pallet does not support the use of any keys other than ed25519 to sign extrinsics. This is because we are using ed25519 keypairs on Zero-OS and we want to make this consistent on chain as well.

Every `address` field noted below in this document describes an ed25119 Public Key in [SS58](https://substrate.dev/docs/en/knowledgebase/advanced/ss58-address-format) format. We use this encoding since it's the Substrate standard.

All the objects depicted in this document are linked to an ed25519 Keypair. Once an object is created with a keypair, only the owner of the Keypair can make changes or delete that object. This way we can be sure that a user cannot change details for an object that does not belong to him.

An exception is made for Nodes; we allow Farmers to change the settings of their Node, since they own the physical hardware and must also be able to change it's digital representation.

## Entities

Entities are links to physical human beings. Only one entity object per person can or must be created in the database. A person will register himself on substrate through the Threefold Connect application.

An Entity object has following fields:

```js
{
    "version": "gridVersion",
    "id": NumericEntityID,
    "name": "PersonName",
    "address": "SubstrateAccountID",
    "country_id": IdOfTheCountry,
    "city_id": IdOfTheCity
}
```

The Entity's name is the physical user's chosen name.

The address (SubstrateAccountID) is set on the moment the Entity is created. It extracts the substrate account ID from the signed request and assigns it the Entity object. From that moment, only the keypair that created this Entity can edit or delete this object afterwards.

Country and City id's are the id's of the country and city that the person lives in. A lookup for these id's can be done on the [Graphql](https://github.com/threefoldtech/vgrid/blob/main/wiki/main/specs/substrate/griddb/graphql.md) instance.

## Twins

Twins are digital copies of humain beings that control:

- Node
- Farm

A Twin object has the following fields:


```js
{
    "version": "gridVersion",
    "id": NumericTwinID,
    "address": "SubstrateAccountID",
    "ip": "ip4/ip6",
    "entities": ListOfEntityRelationshipProofs
}
```

The address (SubstrateAccountID) is set on the moment the Twin is created. It extracts the substrate account ID from the signed request and assigns it the Twin object. From that moment, only the keypair that created this twin can edit or delete this object afterwards.

Twins must set an IP field, this field can either be ipv4/ipv6. Be setting this value, Twins can talk to other remote Twins over a [message bus](https://github.com/threefoldtech/rmb/).

A twin is an anonymous entity in substrate, if a twin wishes to make himself a known enitity he can link up with an Entity. Since an Entity has a `name` field and is linked to a user.

An EntityProof example:

```js
{
    "entity_id": NumericIdOfEntity,
    "signature": SignatureOfEntity (message: "entity_idTwin_id")
}
```

To link an Entity to a Twin object, the Twin object must call `add_twin_entity(..)` on chain. The input being, the target entity and the signature of the entity with message (entity_idTwin_id).

Example:

Twin with ID 1 wants to link up with Entity with ID 5. The target Entity must agree this link, he will have to sign with his ed25519 keypair the following message:

- signEd25519(05)

When this is submitted on chain, the chain will check reconstruct the message and verify the signature with the target entity address.

## Farms

Farmers are digital twins (Twin object) that control a physical Nodes Farm. Before one can construct a Farm object, a Twin must be created. With this Twin's keypair a Farm should be created, this way a Twin and a Farm will be linked to eachother.

A Farmer thus has a digital representation on chain of his physical Farm and Nodes in real life. 

A farmer can have multiple nodes and can set it's prices by linking to a Pricing Policy. (TODO: will be changed to be aligned with pricing policies in the wiki)

If a Farmer has the capability to provide public ip's to his consumers, he can provide a list of ips that are available to any consumer. Public IP's can be added on Farm creation and through `addFarmIp` and `removeFarmIp`. These extrinsics again can only be called by the Farmer's keypair.

A Farm object looks like following on chain:

```js
{
    "version": "gridVersion",
    "id": numericFarmID,
    "name": "FarmName",
    "twin_id": LinkedTwinNumericID,
    "pricing_policy_id" LinkedPricingPolicyNumericID,
    "certification_type": CertificationType(None, Bronze, Silver) TODO ALIGN WITH WIKI,
    "country_id": IdOfTheCountry,
    "city_id": IdOfTheCity
    "public_ips": [PublicIP]
}
```

Public IP Object:

```js
{
    "ip": "someIP4string",
    "gateway": "ip4gateway",
    "contract_id": idOfSmartContract (not set initially)
}
```

## Nodes

A Node is a Twin that control a physical Node. There always has to be a digital reprentation of a physical Node that belongs to a Farmer.

Before a Node object can be registered, a Twin object must be created. Zero-OS running on the Node will be responsible for this. On boot, Zero-OS will generate a keypair for this Node, with that keypair it will register a Twin object on chain. It registers a Twin object with a public ipv4 (if it has that), otherwise an ipv6 will be used.

With this Twin's keypair a Node object should be registered, again Zero-OS is responsible for this. On boot Zero-OS will create a Node object on chain with the proper Farm ID, resources, public config and location. This way Twin and a Node will be linked to eachother.

This Farm ID is passed as a kernel argument on the boot process. When a Farm ID is set, only the Farm with that ID can manage this Node.

When all object are properly registered on chain (Twin and Node). The Node can accept workloads and bill the consumer's wallet accordingly.

A Node object has following fields:

```js
{
    "version": "gridVersion",
    "id": NumericNodeID,
    "farm_id": NumericFarmID,
    "twin_id": NumericTwinID (inferred from the signature of the extrinsic),
    "resources": {
        "sru": totalSru,
        "hru": totalHru,
        "cru": totalCru,
        "mru": totalMru
    },
    "location": {
        "latitude": "someLatValue",
        "longitude": "someLongValue"
    },
    "country_id": IdOfTheCountry,
    "city_id": IdOfTheCity,
    "address": "SubstrateAccountID (same as the Twin who creates this Node)",
    "role": ["Node", "Gateway"],
    "public_config": Optional {
        "ipv4": "publicIP4",
        "ipv6": "publicIP6",
        "gw4": "gatewayip4",
        "gw6": "gatewayip6"
    }
}
```

## Creating / updating / deleting objects

A user can create / update / delete objects on substrate by calling `Extrinsics` on the TfgridModule. Every extrinsic costs some amount of tokens.

Following extrinsics are exposed:

Entities:

- createEntity(..)
- updateEntity(..)
- deleteEntity(..)

Twins:

- create_twin(..)
- update_twin(..)
- delete_twin(..)

Entity-Twin Relation:

- addTwinEntity(..)
- removeTwinEntity(..)

Nodes:

- createNode(..)
- updateNode(..)
- deleteNode(..)

Farms:

- createFarm(..)
- updateFarm(..)
- deleteFarm(..)
- addFarmIp(..)
- removeFarmIp(..)

Every extrinsic must by signed by the user / digital twin that owns or will own the object.

A [cli-tool](https://github.com/threefoldtech/tfgrid-substrate/blob/master/cli-tool/readme.md) can be used to call the extrincis

Or you could use the polkadot UI apps to call extrinsics from the browser:

https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fexplorer.devnet.grid.tf%2Fws#/extrinsics

## Graphql

We store every creation / update / deletion of above objects in a graphql database. An end user of any other application can query the objects from the substrate database without having to talk to the substrate nodes. 

[example](./graphql.md)

## Note

Full type definition can be found [here](https://github.com/threefoldtech/vgrid/blob/main/tfgriddb/tfgriddb_model.v)