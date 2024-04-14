# Documentation

## Development

For information about the development process, see: [development](./development).

## Production

For information about the process to deploy on production, see [production](./production/production.md).

## Testing

To fire all unit tests:

Make sure you have setup your work environment (see [development](./development)).

```bash
cd substrate-node
cargo test
```

For integration tests, see [integration-tests](../substrate-node/tests/readme.md)

## Workflows

- Build and Test: Every commit triggers a build and test process, where tfchain is compiled and all unit and integration tests are executed.

## Release process

See [upgrade-process](./production/upgrade_process.md)
