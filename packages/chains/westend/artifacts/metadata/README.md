## Supported Runtimes
  - Kusama

## Generated files from subxt-cli

Download metadata from a substrate node, for use with `subxt` codegen.

```bash
subxt metadata --url wss://rpc.turboflakes.io:443/westend -f bytes > packages/chains/westend/artifacts/metadata/westend_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/westend --pallets System,Utility,Staking -f bytes > packages/chains/westend/artifacts/metadata/westend_metadata_small.scale
```
