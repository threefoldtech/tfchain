# Documentation

## Development

For information about the development process, see: [development](./development/development.md)

## Production

For information about the process to deploy on production, see [production](./production/production.md)

## Testing

To fire all unit tests:

Make sure you have setup your work environment (see development).

```
cd substrate-node
cargo test
```

For integration tests, see [integration-tests](../substrate-node/tests/readme.md)

## Workflows

- Build and Test: with every commit, tfchain will be built and all tests (unit / integration) tests will run.

## Release process

See [upgrade-process](./production/upgrade_process.md)
