# suno &middot; ![latest release](https://github.com/turboflakes/suno/actions/workflows/create_release.yml/badge.svg)

`suno` -- Just another way of managing **Su**bstrate **n**ode **o**perations in the Polkadot ecosystem.

## Development / Build from Source

If you'd like to build from source, first install Rust.

```bash
curl https://sh.rustup.rs -sSf | sh
```

If Rust is already installed run

```bash
rustup update
```

Verify Rust installation by running

```bash
rustc --version
```

Once done, finish installing the support software

```bash
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```

Build `suno` by cloning this repository

```bash
#!/bin/bash
git clone http://github.com/turboflakes/suno
```

Compile `suno` package with Cargo

```bash
#!/bin/bash
cargo build
```

And then run it

```bash
#!/bin/bash
./target/debug/suno
```

Otherwise, recompile the code on changes and run the binary

```bash
#!/bin/bash
cargo watch -x 'run --bin suno'
```
