# Runtime upgrade module

A wrapper pallet for [frame-system](https://github.com/paritytech/substrate/tree/master/frame/system) specifically the `set_code` action. This action replaces the runtime code with the new code. 

## Config

- `SetCodeOrigin` - The origin that can call the `set_code` action

## Interface

Dispatchable functions of this pallet.

- `set_code` - Set the runtime code (Can only by signed by a configurable Origin in the Config)