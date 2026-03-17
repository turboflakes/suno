# suno &middot; [![latest commit](https://github.com/turboflakes/suno/actions/workflows/rust.yml/badge.svg)](https://github.com/turboflakes/suno/actions/workflows/rust.yml) ![latest release](https://github.com/turboflakes/suno/actions/workflows/create_release.yml/badge.svg)

<p align="center">
  <img src="https://github.com/turboflakes/suno/blob/main/assets/suno-github-header.webp?raw=true">
</p>

`suno` -- Yet another way to manage **Su**bstrate **N**ode **O**perations from your terminal. `suno` is a terminal user interface to monitor live data and manage your own or third-party nodes. It supports Polkadot, Kusama, Paseo, and Westend networks.

## Implementation constraints
 - Runs on the terminal.
 - Users can connect to any RPC node of their choice. 
 - No backend APIs. All displayed data comes directly from the RPCs configured.
 - Restricted Proxy-Only operations with only three proxy types supported:
  - Staking, StakingOperator (Asset Hub)
  - NonTransfer (Relay Chain)
 - Proxy account must be an account exported from PJS with a password already encoded (json format).

## Features

- [&check;] Support Polkadot, Kusama, Paseo and Westend networks all at once on the same view;
- [&check;] General network stats. Block height, era and epoch progress.
- [&check;] Total validators and total nominators (active vs waiting).
- [&check;] Network total staked percentage.
- [&check;] Validator status, identity and Live Points
- [&check;] Total nominators, Total stake, Self stake, Bonded, Unbonding, Unlocked. Display payee.
- [&check;] Active vs Next commission. Current and Queued session keys;
- [&check;] Validate and display proxy type for each stash.
- [&check;] Autocomplete, select or filter commands (extrinsics) based on proxy type context.
- [&check;] Support for `/bond`, `/bond_extra`, `/unbond`, `/rebond`, `/withdraw_unbonded`, `/validate`, `/chill`, `/set_keys`, `/purge_keys`, `/set_keys_async`, `/purge_keys_async`.
- [&check;] Verify and sign call_data. Display and log transaction progress.

## Future / Ideas / Work in Progress

 - [] Custom themes
 - [] Config custom commands
 - [] Pro / Advanced mode to show validators key insight metrics
 - [] Collator metrics and extrinsics
 - [] RPC manual restarts and health check metrics
 - [] Light client mode
 - [] Multi-proxy setup

## Installation
<!-- TODO -->

## Configuration
<!-- TODO -->

## Usage
<!-- TODO -->

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

## Collaboration

Have an idea for a new feature, a fix or you found a bug, please open an [issue](https://github.com/turboflakes/suno/issues) or submit a [pull request](https://github.com/turboflakes/suno/pulls).

Any feedback is welcome.

### License

**suno** - The entire code within this repository is licensed under the [Apache License 2.0](./LICENSE).

__

Enjoy `suno`
