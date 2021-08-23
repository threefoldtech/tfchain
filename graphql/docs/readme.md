# Graphql docs

This document will summarize maintenance of this graphql stack.

Components of this stack:

Indexer stack

- Indexer (indexes substrate events and saves it to the indexer-database)
- Indexer database (sql database where substrate events are stored)
- Indexer-api-gateway (api to query the indexer's status)
- Redis (indexer cache)
- Indexer-status-service (new component that can extract the indexers syncing status and other info)

Processor stack

- Processor (processes indexer events and maps to readable data)
- Processor database (processed events database)
- Query Node (graphql frontend on top of the processor database)

# Indexer stack

This one is very straightforward, in `indexer/` directory there is a docker compose yaml file and a `types.json` file. The indexer relies on following configuration set in the `docker-compose.yaml`:

- WS_PROVIDER_ENDPOINT_URI (substrate chain public url)
- TYPES_JSON filke (custom runtime types definitions)

Run it:

```
docker-compose up -d
```

## Types file

Types file should ALWAYS be aligned with https://github.com/threefoldtech/tfgrid-api-client/blob/master/types.json

## Ws provider endpoints

Substrate chain public Websocket endpoint.

# Processor stack

The processor stack should run seperately from the indexer stack.

## Compile and prepare processor stack

Install dependencies, create the processor db, run migrations and run the init script, run the processor migration and start the stack.

```
yarn
yarn codegen
yarn db:up
yarn db:migrate
yarn db:init
yarn processor:migrate
```

## Configuration

The processor has an environment variable that needs to be set: `INDEXER_ENDPOINT_URL`, which is the Indexer-api-gateway url. If run on the same machine it can reach the indexer-api on the private ip or docker ip. 

Example: `INDEXER_ENDPOINT_URL=http://172.17.0.1:4010/v1/graphql`

## Run it

Finally run the stack:

```
docker-compose up -d
```

Check syncing status:

```
docker logs graphql_processor_1 -f
```

Query node frontend is available on localhost:4000/graphql/


# Upgrade process

check [process](./upgrading.md)