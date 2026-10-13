# pvm-rust

The Polkadot Virtual Machine (PVM) in Rust, implemented from the
[JAM Gray Paper](https://graypaper.com). [Jamixir](https://jamixir.org), an Elixir JAM
client, runs it through a Rustler NIF. The crate itself has no Elixir dependency.

## Layout

- `core`: memory, registers, program and bitmask handling
- `encoding`: program blob decoding (`deblob`) and encoding
- `init`: program initialisation and memory layout
- `vm`: instruction dispatch, single-step execution, gas, child instances, tracing
- `execution`: entry points for disassembled and linked programs

## Tests

```sh
git clone https://github.com/jamixir/jamtestvectors ../pvm-test-vectors
cargo test
```

The unit tests need nothing else. The `vectors` suite reads the PVM test vectors from
`../pvm-test-vectors/pvm/programs/`.

## License

[Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
