# Upgrade graphql types

After you made all the type changes in 

- Runtime modules
- Api client library
- Typescript library

You can continue making the type change in graphql.

## Modify package.json dependencies

Open `package.json` and bump the following dependency to the new typescript library version:

```
...
    "substrate-tfgrid-ts-types": "^0.0.19",
...
```

run `yarn` afterwards.

## Modify graphql schema

Open `schema.grapqhl` and make the type change according to the changed types. Save the file.

## Generate code

Generate code again.

```
yarn codegen
```

## Change mappings

A mapping file contains methods to map an event's data to an object to be saved in the graphql database. If you change a type, the events emitting these types might need a change as well.

If you for example removed a field in the Node object called `uptime` and you edited all the runtime code, api client code, typescript code and graphql schema then you need to open

`mappings/nodes.ts` and remove all references to the `uptime` field. After a `yarn codegen` your editor should also give errors when you open the file that the attribute is not present anymore on the Node object.

Make the changes accordining to all changed types.

## Compile and restart processor

First drop the database and processor services

```
docker-compose down
```

Recompile the processor's docker image

```
docker build . -t processor:latest
```

Restart database:

```
yarn db:up
yarn db:migrate
yarn db:init
```

Restart processor:

```
yarn processor migrate
docker-compose up -d
```

## IMPORTANT NOTES

Type changes are not that easy to handle on a running network with substrate and graphql. There are multiple reasons for this, the most important one is actually maintaining the processor code. The main thing here is actually that when the processor restarts, it starts processing all events from the beginning of the blockchain time until what is running but this can cause type issues.

Because, for example if a node object is stored at block 100 with fields:

```
...
node_id: 1,
uptime: 100,
farm_id: 1,
...
```

And another node object is stored at block 1000 with fields:

```
...
node_id: 1,
farm_id: 1,
...
```

Notice the field `uptime` is gone. The first time the mapper will try to decode the event data into an object that has no `uptime` field (because in this document we described an exdample where we would remove a field). And this decoding will fail because the blockchain storage at block 100 contains such an event with a Node object that still has this field!! 

To work around this you COULD use a different mapper function at block 100 than at block 1000 to work around the type change. This can be done by configuring this in the `manifest.yaml` file. If for example a new runtime is pushed to remove a field from a Node object and there was no storage migration attached you can say for example that a function `mapNodesV2` must be used from spec version 2 (spec version increment at a runtime upgrade). In this way, you can use 2 different mapping functions that will map an "old" object and a "new" object to a graphql db object.

