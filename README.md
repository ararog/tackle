# tackle

[![Crates.io downloads](https://img.shields.io/crates/d/tackle)](https://crates.io/crates/tackle) [![crates.io](https://img.shields.io/crates/v/tackle?style=flat-square)](https://crates.io/crates/tackle) [![Build Status](https://github.com/ararog/tackle/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/ararog/tackle/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/tackle) [![Documentation](https://docs.rs/tackle/badge.svg)](https://docs.rs/tackle/latest/tackle) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/ararog/tackle/blob/main/LICENSE.md)  [![codecov](https://codecov.io/gh/ararog/tackle/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/ararog/tackle)

## Description

Tackle is a library which makes easy for developers plug a hook/interceptor/middleware solution in your async libraries and projects.

## Install

Either run from command line:

`cargo add tackle`

Or add to your `Cargo.toml`:

```toml
tackle = { version = "0.1.0" }
```

## Usage

```rust, compile_fail

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();

}
```

## License

MIT or Apache-2.0

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
