#!/bin/bash
#
# > make a file executable
# chmod +x ./update-chain-specs.sh
#
# > subxt-cli must be installed to update metadata
# cargo install subxt-cli --force --features chain-spec-pruning
#
# Optionally restrict the run to a single network, e.g.
# `update-chain-specs.sh polkadot`
BASE="packages/config"
NETWORK="$1"
SCRIPT_DIR="$(dirname "$0")"

# Where to write the finalized block diff report for this run.
# Override with the CHAIN_SPECS_VERSIONS_FILE env var (e.g. to keep it out of the repo).
VERSIONS_FILE="${CHAIN_SPECS_VERSIONS_FILE:-chain_specs_versions.md}"
: > "$VERSIONS_FILE"

# Prints the block number encoded in a chain spec's finalizedBlockHeader, or
# "unknown" if the file/field is missing or can't be decoded.
finalized_block_number() {
  local file="$1"
  local header
  header=$(jq -r '.lightSyncState.finalizedBlockHeader // empty' "$file" 2>/dev/null)
  [ -z "$header" ] && { echo "unknown"; return; }
  bash "$SCRIPT_DIR/decode-header-block-number.sh" "$header" 2>/dev/null || echo "unknown"
}

fetch_chain_specs() {
  local chain="$1"      # e.g. "westend"
  local host="$2"       # e.g. "westend.rpc.turboflakes.io"

  if [ -n "$NETWORK" ] && [ "$chain" != "$NETWORK" ]; then
    return 0
  fi

  # Derive output filename: replace hyphens with underscores
  local filename="${chain//-/_}.json"
  local out_dir="$BASE/chain-specs"
  local out_file="$out_dir/$filename"

  mkdir -p "$out_dir"

  local old_block
  old_block=$(finalized_block_number "$out_file")

  # Retry fetching metadata up to $max_attempts times
  local max_attempts=3
  local attempt=1
  until subxt chain-spec --url wss://$host:443 --output-file "$out_file" --state-root-hash --remove-substitutes; do
    if [ "$attempt" -ge "$max_attempts" ]; then
      echo "ERROR: failed to fetch metadata for $chain after $max_attempts attempts"
      return 1
    fi
    echo "Attempt $attempt failed for $chain, retrying in 10s..."
    attempt=$((attempt + 1))
    sleep 10
  done

  local new_block
  new_block=$(finalized_block_number "$out_file")

  local delta="n/a"
  if [[ "$old_block" =~ ^[0-9]+$ && "$new_block" =~ ^[0-9]+$ ]]; then
    delta=$((new_block - old_block))
  fi

  {
    echo "✦ Chain specs for $chain"
    echo "- Finalized block #$new_block (Δ$delta blocks)"
  } >> "$VERSIONS_FILE"
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
