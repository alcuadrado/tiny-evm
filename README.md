# tiny-evm

A tiny EVM implementation in Rust.

## What's included

This crate only implements the core execution engine of the EVM.

## What's not included

This crate doesn't implements:

* Gas metering nor out of gas errors

* The Ethereum world state

* Inter-account calls

* Any kind of hardfork-specific logic

## TODO

* [ ] Publish it to crates.io

* [ ] Create wasm bindings and publish an npm package

* [ ] Create an N-API bindings and publish an npm package

## License

MIT
