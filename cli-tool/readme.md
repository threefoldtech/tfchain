# Substrate TF Grid cli tool

## Installing

`yarn`

## Environment setup

For ease of use you can create a `.env` file in the root of this project with following config:

```
SUBSTRATE_API_URL=someURl
MNEMONIC=someMnemonic
```

This will load these credentials for every command you execute.

## Getting started

check `node cli.js -h` for available commands.

## Entities

### Get an Entity

```
node cli.js entity get <id>
```