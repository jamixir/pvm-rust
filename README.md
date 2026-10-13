# pvm-rust

A Rust implementation of the Polkadot Virtual Machine (PVM), the RISC-V-based VM that runs
JAM services, written from the [JAM Gray Paper](https://graypaper.com).

This is the PVM inside [Jamixir](https://jamixir.com), the Elixir JAM client that was
awarded JAM Prize Milestone 1. The node calls it through a Rustler NIF; this crate is the
VM itself and has no Elixir dependency.

## What's in it

- `core`: memory, registers, program and bitmask handling
- `encoding`: program blob decoding (`deblob`) and encoding
- `init`: standard program initialisation and memory layout
- `vm`: instruction dispatch, single-stepping, gas accounting, child VM instances, tracing
- `execution`: entry points for running disassembled or linked programs

## Tests

```sh
git clone https://github.com/jamixir/jamtestvectors ../pvm-test-vectors
cargo test
```

Unit tests run on their own. The `vectors` suite runs the PVM test vectors and expects them
at `../pvm-test-vectors/pvm/programs/`; without that checkout those tests fail.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.
