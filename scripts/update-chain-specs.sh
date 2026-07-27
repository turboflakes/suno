#!/bin/bash
#
# > make a file executable
# chmod +x ./update-chain-specs.sh
#
# > subxt-cli must be installed to update metadata
# cargo install subxt-cli --force

BASE="../packages/config"

fetch_chain_specs() {
  local chain="$1"      # e.g. "westend"
  local host="$2"       # e.g. "westend.rpc.turboflakes.io"

  # Derive output filename: replace hyphens with underscores
  local filename="${chain//-/_}.json"
  local out_dir="$BASE/chain-specs"

  mkdir -p "$out_dir"
  subxt chain-spec --url wss://$host:443 --output-file "$out_dir/$filename" --state-root-hash --remove-substitutes
}

# Relay Chains
fetch_chain_specs "westend"  "westend.rpc.turboflakes.io"
fetch_chain_specs "paseo"    "paseo.rpc.turboflakes.io"
fetch_chain_specs "kusama"   "kusama.rpc.turboflakes.io"
fetch_chain_specs "polkadot" "polkadot.rpc.turboflakes.io"

# Generate relay chain specs from subxt to be used under lightclient

# ```bash
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/westend --output-file artifacts/demo_chain_specs/westend.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/kusama --output-file artifacts/demo_chain_specs/kusama.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/polkadot --output-file artifacts/demo_chain_specs/polkadot.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/paseo --output-file artifacts/demo_chain_specs/paseo.json --state-root-hash --remove-substitutes
# ```
