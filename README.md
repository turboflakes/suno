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
- [&check;] Add builtin themes [`Suno Dark`, `Suno Light`] and load user specific **custom themes**.

## Future / Ideas / Work in Progress
 - [] Config custom commands
 - [] Pro / Advanced mode to show validators key insight metrics
 - [] Collator metrics and extrinsics
 - [] RPC manual restarts and health check metrics
 - [] Light client mode
 - [] Multi-proxy setup
 - [] Support for `/kick`, `/nominate` extrinsics
 
## Installation

**Note: Binary release available for Linux and macOS only**

### Option 1
Download and extract the latest binary from the GitHub [Releases](https://github.com/turboflakes/suno/releases) section.

### Option 2
Alternatively, run the bash script in [/scripts/install.sh](https://raw.githubusercontent.com/turboflakes/suno/refs/heads/main/scripts/install.sh) with the command below:

```bash
curl -fsSL https://raw.githubusercontent.com/turboflakes/suno/main/scripts/install.sh | bash
```
The script downloads the latest release, extracts it, and installs the binary into the `$HOME/suno` directory by default. It prompts you to change the directory as well as asking if a default configuration file is required.

An example of the instructions presented:
```bash
> Enter SUNO installation path [default: /home/suno]:
√ Output directory /home/suno
√ Downloading suno v0.1.2
suno-aarch64-apple-darwin.tar.gz          100%[=====================================================================================>]  21.12M  17.3MB/s    in 1.2s
suno-aarch64-apple-darwin.tar.gz.sha256   100%[=====================================================================================>]      99  --.-KB/s    in 0s
√ Checksum verified
√ Existing binary backed up to /home/suno/suno.backup
√ Checking if suno exists: total 262688
-rwxr-xr-x@ 1 paulo  staff  112007760 Mar 17 12:05 suno
-rw-r--r--@ 1 paulo  staff   22151830 Mar 17 12:06 suno-aarch64-apple-darwin.tar.gz
-rw-r--r--@ 1 paulo  staff         99 Mar 17 12:06 suno-aarch64-apple-darwin.tar.gz.sha256
√ Successfully installed suno v0.1.2 at /home/suno/suno
> Would you like to install the DEFAULT configuration file? [y/N]: y
> Enter the configuration path [default: /Users/paulo/suno/.config.yaml]:
√ Writing /Users/paulo/suno/.config.yaml template
√ Config file saved at /Users/paulo/suno/.config.yaml.
-> Edit the config file and replace STASHES and RPC endpoints as you wish.
√ Installation complete
— Enjoy suno v0.1.2
```

## Configuration

### Validator **stashes** and **RPCs**
Most configuration is done via an initialized config file. Here is a full example [.config.example.yaml](https://raw.githubusercontent.com/turboflakes/suno/refs/heads/main/.config.example.yaml), showing all available options:

```yaml
chains:
  - polkadot:
      rpc_url: "wss://polkadot.rpc.PROVIDER_ENDPOINT"
      light_client: false
      validators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
  - asset_hub_polkadot:
      rpc_url: "wss://asset-hub-polkadot.rpc.PROVIDER_ENDPOINT"
      light_client: false
  - people_polkadot:
      rpc_url: "wss://people-polkadot.rpc.PROVIDER_ENDPOINT"
      light_client: false
  - kusama:
      rpc_url: "wss://kusama.rpc.PROVIDER_ENDPOINT"
      light_client: false
      validators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
  - asset_hub_kusama:
      rpc_url: "wss://asset-hub-kusama.rpc.PROVIDER_ENDPOINT"
      light_client: false
features:
  enable_validators: true
  enable_collators: true
  enable_rpcs: false
themes:
  active: "Suno Dark"
  path: "./themes"
signer:
  proxy_path: ".proxy_private.json"
explorer:
    url: "https://polkadot.js.org/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
    # A few other explorers commented out below
    # url: "https://dev.papi.how/explorer/{block_hash}#networkId=localhost&endpoint=wss://{chain}.rpc.turboflakes.io"
    # url: "https://polkadot.chainconsole.com/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
```

## Signer Account (Proxy-Only)
To operate and execute extrinsics onchain, a proxy account with at least one of the following types `Staking`, `StakingOperator`, `NonTransfer` must be set-up for the stashes listed in the configuration file. For example, `Staking` (short form as visualized in the tool `[S]`) or `StakingOperator` [SO] must be setup on the Asset-Hub chain, and `NonTransfer` [NT] on the Relay chain.

###  Commands supported per Proxy Type
  NOTE: Each `suno` command is intrinsically dependant on its availability within the runtime

- **[S] Staking (Asset Hub)**
  - `/bond`
  - `/bond_extra`
  - `/unbond`
  - `/rebond`
  - `/withdraw_unbonded`
  - `/set_payee`
  - `/validate`
  - `/chill`

- **[SO] StakingOperator (Asset Hub)**
  - `/validate`
  - `/chill`
  - `/set_keys_async`
  - `/purge_keys_async`
  
- **[NT] NonTransfer (Relay Chain)**
  - `/set_keys`
  - `/purge_keys`

### Proxy Account configuration

#### Step 1
Currently, to setup the proxy account on `suno`, the ONLY supported, recommended and easiest way, is to create a new account on the [Polkadot Developer Signer](https://polkadot.js.org/extension/) and than click **Export Account**. You should get a json file with the content similar to the one below:
```json
{
  "encoded": "J2FFcPHAY11Pmq/38eqbwfUv9OPitYJs+oYgahBvlagAAAIAAQAAAAgAAAB5o0DwXCWDblsH+9pc++RaBO4fpHBHzUirHFHFE9yS3sDzgAIQjhgvPqJ3ODrMR2gy7vk0VZg1fyirIvmsrfjGbWnOI8YU0joX0tYytroyWaykFKtZJMmE0pNKcJ5dJmDxscbK53Ac+7ld2UdH07yKPXxmPuYNNw3vKx8cg9CdQgifKfzQxHnC+EUpOoHPLwGlHsFEYtIlQtngqd9n",
  "encoding": {
    "content": ["pkcs8", "sr25519"],
    "type": ["scrypt", "xsalsa20-poly1305"],
    "version": "3"
  },
  "address": "5CfWTDh7XxJ2yrayqQ2aJnnZAH5v5XaF1oJFfH5QCpbfP9v8",
  "meta": {
    "genesisHash": "",
    "name": "Bob",
    "whenCreated": 1768916488918
  }
}
```

#### Step 2
You can rename the file to `.proxy_private.json`, since it is the one built-in by default and is expected to live alongside the binary. Alternatively, you can rename it and move the file to a directory of your choice. If you choose a different name and path, you have 2 options:

**Option 1** specify the new **proxy_path** under the **signer** section in the configuration file. For example:

```yaml
signer:
  proxy_path: ".proxy_private.json"
```

**Option 2** via the `--proxy-path` flag when calling `suno` from the terminal, eg. `suno --proxy-path /home/suno/suno-proxy-account.json`

## Usage
From your favourite terminal, simply call `suno`. If you use a custom configuration file, located in a different directory than the `suno` binary, provide the path with the `--config-path` flag, eg. `suno --config-path ~/suno/suno-custom-config.json`

Check all flags available:

```bash
suno --help
Yet another way to manage Substrate Node Operations from your terminal.

Usage: suno [OPTIONS]

Options:
  -c, --config-path <FILE>  Sets a custom config file path. [default: .config.yaml]
  -p, --proxy-path <FILE>   Sets a custom proxy account file path.
  -h, --help                Print help
  -V, --version             Print version
```

### List of keybindings

```bash
'ctrl+w' to switch window
'ctrl+c' to quit suno
'esc' to close popup or unfocus from input field
'ctrl+h / ctrl+l / left / right' to navigate between pane sections
'tab' to input focus, to command autocomplete or just to navigate between pane sections
'ctrl+j / ctrl+k / up / down' to select a chain, validator, or extrinsic depending on the highlighted area
```

### Change or Build your own **theme**

The `Suno Dark` and `Suno Light` themes are built-in, you can swap between them by updating the configuration file.

To create your own **theme**, pick one of the ones available in the [/themes](https://github.com/turboflakes/suno/tree/main/themes) directory, copy and rename it, adjust the colors as you please. The filename will serve as the theme name. Under the **themes** section in the configuration file (see below), specify the new theme name and adjust the directory path as needed; It should point to the custom themes folder.

```yaml
themes:
  active: "Blue Sky"
  path: "./themes"
```

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
