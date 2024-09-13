# rusty-chunkenc

[![Crates.io](https://img.shields.io/crates/v/rusty-chunkenc.svg)](<https://crates.io/crates/rusty-chunkenc>)
[![Documentation](https://docs.rs/rusty-chunkenc/badge.svg)](https://docs.rs/rusty-chunkenc)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A Rust implementation of Prometheus' [`chunkenc`](https://pkg.go.dev/github.com/prometheus/prometheus/tsdb/chunkenc) library.

## Features

- Parse Prometheus XOR-encoded chunks (that are heavily inspired by [Gorilla](https://www.vldb.org/pvldb/vol8/p1816-teller.pdf)).
- Serialise time series to Prometheus XOR-encoded chunks.
- Read Prometheus' cold data directly from the disk.
- Also comes with utilities to read and write `varint`, `uvarint`, `varbit`, `varbit_ts`, and `varbit_xor` numbers.

## Why?

Prometheus uses XOR Chunks in its remote read API, and I wanted to understand how they work in detail. This crate enables [SensApp](https://github.com/sintef/sensapp) to stream data to Prometheus. SensApp is written in Rust, and I wanted a chunkenc Rust implementation.

Also, writing a parser and a serialiser did sound fun.

## License

Apache 2.0. Check the `LICENSE` file for more details.

## Contributing

Feel free to report issues, contribute, or ask questions about this project.

## Acknowledgements

This project is ported from Prometheus' [`chunkenc`](https://pkg.go.dev/github.com/prometheus/prometheus/tsdb/chunkenc), that used [`go-tzs`](https://github.com/dgryski/go-tsz), that is based on the [Gorilla](https://www.vldb.org/pvldb/vol8/p1816-teller.pdf) paper. The parsing heavily relies on [`nom`](https://crates.io/crates/nom).

The project supports the [Smart Building Hub](https://smartbuildinghub.no/) research infrastructure project, which is funded by the [Norwegian Research Council](https://www.forskningsradet.no/).
