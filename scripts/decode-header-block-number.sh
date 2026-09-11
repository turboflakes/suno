#!/bin/bash
#
# Decodes the block number from a SCALE-encoded Substrate block header, as
# found in a chain spec's lightSyncState.finalizedBlockHeader.
#
# Usage: decode-header-block-number.sh <hex-header>
#        echo <hex-header> | decode-header-block-number.sh
set -euo pipefail

hex="${1:-$(cat)}"
hex="${hex#0x}"

# Reverses the byte order (pairs of hex chars) of a little-endian hex string.
reverse_hex_bytes() {
  local s="$1" out="" i
  for ((i = ${#s} - 2; i >= 0; i -= 2)); do
    out+="${s:$i:2}"
  done
  echo "$out"
}

# Header layout: parentHash (32 bytes) is followed by the compact-encoded
# block number.
offset=64 # 32 bytes of parentHash, in hex chars
first_byte=$((16#${hex:$offset:2}))
mode=$((first_byte & 3))

case "$mode" in
  0)
    number=$((first_byte >> 2))
    ;;
  1)
    le=$(reverse_hex_bytes "${hex:$offset:4}")
    number=$(( (16#$le) >> 2 ))
    ;;
  2)
    le=$(reverse_hex_bytes "${hex:$offset:8}")
    number=$(( (16#$le) >> 2 ))
    ;;
  3)
    length=$(((first_byte >> 2) + 4))
    le=$(reverse_hex_bytes "${hex:$((offset + 2)):$((length * 2))}")
    number=$((16#$le))
    ;;
esac

echo "$number"
