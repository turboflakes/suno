#!/bin/bash
#
# > make a file executable
# chmod +x ./update-metadata.sh
#
# > subxt-cli must be installed to update metadata
# cargo install subxt-cli --force

BASE="../packages/chains"
RC_PALLETS="Session,StakingAhClient,Proxy,Babe,ParasShared"
AH_PALLETS="System,Balances,Proxy,Staking,StakingRcClient,Utility,NominationPools"
PEOPLE_PALLETS="Identity"

fetch_metadata() {
  local chain="$1"      # e.g. "westend", "asset-hub-westend", "people-westend"
  local host="$2"       # e.g. "westend.rpc.turboflakes.io"
  local pallets="$3"

  # Derive output filename: replace hyphens with underscores
  local filename="${chain//-/_}_metadata_small.scale"
  local out_dir="$BASE/$chain/artifacts/metadata"

  mkdir -p "$out_dir"
  subxt metadata --url "wss://$host:443" --pallets "$pallets" -f bytes > "$out_dir/$filename"
}

# Relay Chains
fetch_metadata "westend"  "westend.rpc.turboflakes.io"  "$RC_PALLETS"
fetch_metadata "paseo"    "paseo.rpc.turboflakes.io"    "$RC_PALLETS"
fetch_metadata "kusama"   "kusama.rpc.turboflakes.io"   "$RC_PALLETS"
fetch_metadata "polkadot" "polkadot.rpc.turboflakes.io" "$RC_PALLETS"

# AssetHub Chains
fetch_metadata "asset-hub-westend"  "asset-hub-westend.rpc.turboflakes.io"  "$AH_PALLETS"
fetch_metadata "asset-hub-paseo"    "asset-hub-paseo.rpc.turboflakes.io"    "$AH_PALLETS"
fetch_metadata "asset-hub-kusama"   "asset-hub-kusama.rpc.turboflakes.io"   "$AH_PALLETS"
fetch_metadata "asset-hub-polkadot" "asset-hub-polkadot.rpc.turboflakes.io" "$AH_PALLETS"

# People Chains
fetch_metadata "people-westend"  "people-westend.rpc.turboflakes.io"  "$PEOPLE_PALLETS"
fetch_metadata "people-paseo"    "people-paseo.rpc.turboflakes.io"    "$PEOPLE_PALLETS"
fetch_metadata "people-kusama"   "people-kusama.rpc.turboflakes.io"   "$PEOPLE_PALLETS"
fetch_metadata "people-polkadot" "people-polkadot.rpc.turboflakes.io" "$PEOPLE_PALLETS"

# Generate runtime API client code from metadata.

# ```bash
# subxt codegen --url wss://rpc.turboflakes.io:443/westend | rustfmt --edition=2018 --emit=stdout > westend_metadata.rs
# subxt codegen --url wss://asset-hub-westend.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > asset_hub_westend_runtime.rs
# subxt codegen --url wss://rpc.turboflakes.io:443/kusama | rustfmt --edition=2018 --emit=stdout > kusama_runtime.rs
# subxt codegen --url wss://asset-hub-paseo.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > asset_hub_paseo_runtime.rs
# subxt codegen --url wss://paseo.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > paseo_runtime.rs
# subxt codegen --url wss://polkadot.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > polkadot_runtime.rs
# subxt codegen --url wss://sys.turboflakes.io:443/people-kusama | rustfmt --edition=2018 --emit=stdout > people_kusama_runtime.rs
# subxt codegen --url wss://sys.turboflakes.io:443/people-polkadot | rustfmt --edition=2018 --emit=stdout > people_polkadot_runtime.rs
# subxt codegen --url wss://sys.turboflakes.io:443/people-paseo | rustfmt --edition=2018 --emit=stdout > people_paseo_runtime.rs
# subxt codegen --url wss://asset-hub-polkadot.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > packages/chains/asset-hub-polkadot/artifacts/metadata/asset_hub_polkadot_metadata_small.rs
# ```

# Generate relay chain specs from subxt to be used under lightclient

# ```bash
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/westend --output-file artifacts/demo_chain_specs/westend.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/kusama --output-file artifacts/demo_chain_specs/kusama.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/polkadot --output-file artifacts/demo_chain_specs/polkadot.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/paseo --output-file artifacts/demo_chain_specs/paseo.json --state-root-hash --remove-substitutes
# ```
