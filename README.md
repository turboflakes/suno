# suno &middot; [![latest commit](https://github.com/turboflakes/suno/actions/workflows/rust.yml/badge.svg)](https://github.com/turboflakes/suno/actions/workflows/rust.yml) ![latest release](https://github.com/turboflakes/suno/actions/workflows/create_release.yml/badge.svg)

<p align="center">
  <img src="https://github.com/turboflakes/suno/blob/main/assets/suno-github-header.webp?raw=true">
</p>

`suno` -- Yet another way to manage **Su**bstrate **N**ode **O**perations from your terminal. `suno` is a terminal user interface to monitor live data and manage your own or third-party nodes. It supports [Polkadot](https://polkadot.com/), [Kusama](https://kusama.network/), [Paseo](https://paseo.site/), and Westend networks.

## Why use `suno`

As a node operator, to manage your own or third-party nodes from the terminal.

As a nominator you can just as easily check the validators you nominate.

<p align="center">
    <img alt="suno-dark" src="https://github.com/turboflakes/suno/blob/main/assets/suno-dark.gif?raw=true" />
</p>

## Features
- [&check;] Support Polkadot, Kusama, Paseo and Westend networks all at once on the same view;
- [&check;] General network stats. Block height, era and epoch progress.
- [&check;] Total validators and total nominators (active vs waiting).
- [&check;] Network total staked percentage.
- [&check;] Validator status, identity and **Live Points**
- [&check;] Total nominators, Total stake, Self stake, Bonded, Unbonding, Unlocked. Display payee.
- [&check;] Active vs Next commission. Current and Queued session keys;
- [&check;] Validate and display only supported proxy type for each stash.
- [&check;] Autocomplete, select or filter commands (extrinsics) based on proxy type context.
- [&check;] Support [commands](#commands-supported-per-proxy-type) for most of **Staking Operations** as well as **Rotate session keys**.
- [&check;] Verify and sign call_data. Display and log transaction progress.
- [&check;] Add builtin themes [`Suno Dark`, `Suno Light`] and load user specific **custom themes**.
- [&check;] Define and **run** user-specific commands linked to each configured validator.
- [&check;] Explicitly **use** advanced builtin commands: `calls/rotate_and_set_keys`, `calls/has_keys`, `calls/has_queued_keys`.
- [&check;] Execute custom commands locally or remotely.
- [&check;] Sign transactions using **Polkadot Vault**.
- [&check;] Support **Multi-proxy** setup

## Future / Ideas / Work in Progress
 - [] Pro / Advanced mode to show validators key insight metrics
 - [] Collator metrics and extrinsics
 - [] RPC manual restarts and health check metrics
 - [] Light client mode
 - [] Support for `/kick`, `/nominate` extrinsics

### Implementation constraints / goals
 - Runs on the terminal.
 - No backend APIs. No indexers.
 - Users are free to swap between any RPC node provider of their choice. Connect to Local, Private or Public nodes.
 - Restricted Proxy-Only operations on Asset Hub, with only two proxy types supported:
    - Staking or StakingOperator
 
## Installation

**Note: Binary release available for Linux and macOS**

### Option 1
Download and extract the latest binary from GitHub [Releases](https://github.com/turboflakes/suno/releases).

### Option 2
Alternatively, run the bash script in [/scripts/install.sh](https://raw.githubusercontent.com/turboflakes/suno/refs/heads/main/scripts/install.sh) with the command below:

```bash
curl -fsSL https://raw.githubusercontent.com/turboflakes/suno/main/scripts/install.sh | bash
```
The script downloads the latest release, extracts it, and installs the binary into the `$HOME/suno` directory by default. It prompts you to change the directory as well as asking if a default configuration file is required.

An example of the instructions presented:
```bash
> Enter SUNO installation path [default: /home/paulo/suno]: 
✔︎ Output directory /home/paulo/suno
✔︎ Downloading suno v0.2.0
suno-x86_64-unknown-linux-gnu.tar.gz        100%[==================================================================>]  18.01M  25.7MB/s    in 0.7s    
suno-x86_64-unknown-linux-gnu.tar.gz.sha256 100%[==================================================================>]     103  --.-KB/s    in 0s      
✔︎ Checksum verified
✔︎ Existing binary backed up to /home/paulo/suno/suno.backup
✔︎ Checking if suno exists: total 89896
-rwxr-xr-x 1 paulo paulo 73157832 Mar 19 14:52 suno
-rw-rw-r-- 1 paulo paulo 18887761 Mar 19 15:42 suno-x86_64-unknown-linux-gnu.tar.gz
-rw-rw-r-- 1 paulo paulo      103 Mar 19 15:42 suno-x86_64-unknown-linux-gnu.tar.gz.sha256
✔︎ Successfully installed suno v0.2.0 at /home/paulo/suno/suno
> Would you like to install the DEFAULT configuration file? [y/N]: y
> Enter the configuration path [default: /home/paulo/suno/.config.yaml]: 
✔︎ Writing /home/paulo/suno/.config.yaml template
✔︎ Config file saved at /home/paulo/suno/.config.yaml. 
==> Next edit the config file and replace STASHES and RPC endpoints as you wish.
✔︎ Installation complete
— Enjoy suno v0.2.0
```

## Configuration

### Validator **stashes** and **RPCs**
Most configuration is done via a config file. Here is a full example [.config.example.yaml](https://raw.githubusercontent.com/turboflakes/suno/refs/heads/main/.config.example.yaml), showing all available options:

```yaml
chains:
  - polkadot:
      rpc_url: "__WSS_POLKADOT_RPC_PROVIDER__"
      validators:
        - "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        - "1LfAfKweyPjXs4JkKW4AxHPTe7pu4w4HjcZbEtB6a8vMqkd"
        
  - asset_hub_polkadot:
      rpc_url: "__WSS_POLKADOT_HUB_RPC_PROVIDER__"
      signer:
        proxy_account: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        
  - people_polkadot:
      rpc_url: "__WSS_POLKADOT_PEOPLE_RPC_PROVIDER__"
      
  - kusama:
      rpc_url: "__WSS_KUSAMA_RPC_PROVIDER__"
      validators:
        - stash: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
          ssh: # optional, if configured all commands are executed via ssh, otherwise local
            host: 192.0.2.100
            user: suno_user
            # port: 22 # optional, default 22
            # identity: "~/.ssh/id_ed25519" # optional, falls back to SSH agent
          commands:
            - name: Rotate and Set keys
              uses: "calls/rotate_and_set_keys"

            - name: Has session keys
              uses: "calls/has_keys"

            - name: Has queued session keys
              uses: "calls/has_queued_keys"

            - name: Restart service
              cmd: /restart
              run: systemctl restart the-node-01.service

            - name: Upgrade node binary
              cmd: /upgrade {version}
              run: ~/update_stable_node.sh {version}
              
  - asset_hub_kusama:
      rpc_url: "__WSS_KUSAMA_HUB_RPC_PROVIDER__"
      
  - paseo:
      rpc_url: "__WSS_PASEO_RPC_PROVIDER__"
      validators:
        [
          "1LfAfKweyPjXs4JkKW4AxHPTe7pu4w4HjcZbEtB6a8vMqkd",
          "13iiwNL7mzjuS4KxXEHQ2Csx8fETishXWKzeeCfHCig6j2dd",
        ]
  - asset_hub_paseo:
      rpc_url: "__WSS_PASEO_HUB_RPC_PROVIDER__"
      signer:
        proxy_account: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
        
  - people_paseo:
      rpc_url: "__WSS_PASEO_PEOPLE_RPC_PROVIDER__"

features:
  enable_validators: true
  
themes:
  active: "Suno Dark"
  path: "./themes"

signer: # global signer configuration, overridden by chain-specific signer config
  proxy_path: ".proxy_account.json"
  
explorer:
    url: "https://polkadot.js.org/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
    # A few other explorers commented out below
    # url: "https://dev.papi.how/explorer/{block_hash}#networkId=localhost&endpoint=wss://{chain}.rpc.turboflakes.io"
    # url: "https://polkadot.chainconsole.com/apps/?rpc=wss://{chain}.rpc.turboflakes.io#/explorer/query/{block_hash}"
```

## Signer Account (Proxy-Only)
To operate and execute extrinsics onchain, a proxy account with at least one of the following types `Staking`, `StakingOperator` must be set-up for the stashes listed in the configuration file. For example, `Staking` (short form as visualized in the tool `[S]`) or `StakingOperator` `[SO]` must be setup on the Asset-Hub chain.

### Commands supported per Proxy Type
  NOTE: Each `suno` command is intrinsically dependant on its availability within the runtime

- **[S] Staking**
  - `/bond`
  - `/bond_extra`
  - `/unbond`
  - `/rebond`
  - `/withdraw_unbonded`
  - `/set_payee`
  - `/validate`
  - `/chill`
  - `/set_keys`
  - `/purge_keys`
  - `/rotate_and_set_keys`¹

- **[SO] StakingOperator**
  - `/validate`
  - `/chill`
  - `/set_keys`
  - `/purge_keys`
  - `/rotate_and_set_keys`¹
  
¹ Requires explicit configuration for each validator in the config file.

### Signing and Multi-Proxy Accounts configuration

You have two options for signing transactions: either by using Polkadot Vault (recommended) with air-gapped QR codes, or by using a proxy account configured in a local JSON file exported from [PJS](https://polkadot.js.org/extension/).

You can also configure multiple proxy accounts for different chains using the signer section in the configuration file, or configure a single proxy account that is shared across all chains.

#### Option 1
The `proxy_account` field accepts a proxy account address, allowing `suno` to sign transactions via Polkadot Vault. `suno` generates the call data as QR codes and requests access to your webcam to scan the signed QR code from Polkadot Vault.

Add the signer section to your config file as a global setting or as a chain-specific setting:
```yaml
signer:
  proxy_account: "5CfWTDh7XxJ2yrayqQ2aJnnZAH5v5XaF1oJFfH5QCpbfP9v8"
```

Alternatively, you can specify a global proxy account using the `--proxy-account` flag when launching suno from the terminal. For example:
```bash
suno --proxy-account 5CfWTDh7XxJ2yrayqQ2aJnnZAH5v5XaF1oJFfH5QCpbfP9v8
```

#### Option 2
Create a new account on the [PJS](https://polkadot.js.org/extension/) and than click **Export Account**. This will generate a JSON file similar to the one below:
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

You can rename the file to `.proxy_account.json`, which is the default filename expected by `suno` and this file should reside alongside the `suno` binary.

Alternatively, you can rename the file and move it to a directory of your choice. Specify its path using the **proxy_path** field under the **signer** section in the configuration file. For example:

```yaml
signer:
  proxy_path: ".proxy_account.json"
```

You can also specify the global proxy account file using the `--proxy-path` flag when calling `suno` from the terminal. For example:
```bash
suno --proxy-path /home/suno/suno-proxy-account.json
```

**NOTE:** If you create a brand new account, don't forget to transfer some funds to it and configure the appropriate proxy types for the target stash accounts you want to manage with `suno`. This can be done using any other tool in the Polkadot ecosystem that supports proxy management.

## Custom Commands

Custom commands are user defined commands or a composition of builtin commands. These are defined in the `config.yaml` file and tied to each configured `stash`. 

There are two types of custom commands:

#### 1. **_Shell_**
User defined commands that are executed on the terminal, can call simple shell commands or bash script files, basically any program that can run in the terminal. As long as the configured commands e.g. `/service_restart`, `/upgrade`, `/reboot` do not clash with existing ones, they can be named anything. 

####  2. **_Uses_**
Custom calls that are builtin, but is up to the user to enable them. These can be a composition of extrinsics and RPC calls, e.g. a unique command to rotate session keys and automatically set those keys, or a simple command to check if the X host has the current session keys. Currently these special commands are: 

- Use `calls/rotate_and_set_keys` to rotate and set session keys as a single operation (requires a proxy to be already set up).
- Use `calls/has_keys` to check whether the host has the next session keys.
- Use `calls/has_queued_keys` to check whether the host has the queued session keys.

**NOTE**: `curl` must be available on the host machine for RPC calls to execute successfully.

Below is how you can define custom commands in the `config.yaml`:

```
    validators:
        - stash: "5GTD7ZeD823BjpmZBCSzBQp7cvHR1Gunq7oDkurZr9zUev2n"
          host_rpc: 127.0.0.1:9944 # optional, used in curl RPC calls and falls back to 127.0.0.1:9944 

          ssh: # optional, if configured all commands are executed via SSH, otherwise local
            host: 192.0.2.100
            user: suno_user
            # port: 22 # optional, default 22
            # identity: "~/.ssh/id_ed25519" # optional, falls back to SSH agent

          commands:
            - name: "Test custom commands"
              cmd: /echo
              run: "echo 'hello world' > hello.txt"

            - name: "Restart service"
              cmd: /restart
              run: "systemctl restart the-node-01.service"

            - name: "Rotate and set keys"
              uses: "calls/rotate_and_set_keys"
```

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
'ctrl+e' to show list of enabled commands for the selected vaidator
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

When you are done, make a [PR](https://github.com/turboflakes/suno/pulls) with your art, I'll thank you :)

<p align="center">
    <img alt="suno-light" src="https://github.com/turboflakes/suno/blob/main/assets/suno-light.gif?raw=true" />
</p>

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
