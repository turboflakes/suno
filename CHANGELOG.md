# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- Add support for Polkadot-Vault see PR [#20](https://github.com/turboflakes/suno/pull/20)
- Update paseo genesis hash
- Add Multi-proxy setup, different proxy for each network
- Add keybinding `ctrl+m` to mask or unmask hosts, see PR [#21](https://github.com/turboflakes/suno/pull/21)
- Add validator table columns configuration, see PR [#22](https://github.com/turboflakes/suno/pull/22)
- Update metadata polkadot/2003000
- Update metadata asset-hub-polkadot/2003001
- Update metadata people-polkadot/2003000
- Update metadata kusama/2003000
- Update metadata asset-hub-kusama/2003000
- Update metadata people-kusama/2003000
- Update metadata paseo/2003001
- Update metadata asset-hub-paseo/2004000
- Update metadata people-paseo/2004000
- Update metadata westend/1024001
- Update metadata asset-hub-westend/1024001
- Update metadata people-westend/1024001


## [0.4.1] - 2026-06-02
- Review `has_keys` and `has_queued_keys` descriptions and only show next keys when set

## [0.4.0] - 2026-06-01
- Add support for custom commands defined in the configuration file `config.yaml`. Read implemendation details at [PR7](https://github.com/turboflakes/suno/pull/7)
- Implement _Proof_ support for `/set_keys` on all networks
- Remove `/set_keys_async` and `/purge_keys_async`. All staking operations are now on Asset Hub by default, meaning `/set_keys` and `/purge_keys` are executed on AH,
- NonTransfer proxy on the Relay chain is no longer required.
- The validator `host` is now visible when custom commands are configured.
- Update metadata polkadot/2002001
- Update metadata asset-hub-polkadot/2002001
- Update metadata people-polkadot/2001002
- Update metadata kusama/2002000
- Update metadata asset-hub-kusama/2002000
- Update metadata people-kusama/2002000
- Update metadata paseo/2002002
- Update metadata asset-hub-paseo/2002002
- Update metadata people-paseo/2002002
- Update metadata westend/1022003
- Update metadata asset-hub-westend/1022005
- Update metadata people-westend/1022003

## [0.3.0] - 2026-03-23
- Add support for StakingOperator and make commands `/set_keys_async`, `/purge_keys_async` available
- Update metadata polkadot/2001001
- Update metadata asset-hub-polkadot/2001001
- Update metadata people-polkadot/2001001
- Update metadata westend/1022001
- Update metadata asset-hub-westend/1022001
- Update metadata people-westend/1022001
- 
## [0.2.0] - 2026-03-19
- Add support for default builtin themes [`Suno Dark`, `Suno Light`] and user specific Custom Themes.
- Add CLI support for custom `--config-path` and `--proxy-path`.
- Change explorer configuration section to support only `url`.
- Add `install.sh` script to download and install latest version with optional default configuration.

## [0.1.1] - 2026-03-17
- Support Polkadot, Kusama, Paseo and Westend networks all at once on the same view;
- General network stats. Block height, era and epoch progress.
- Total validators and total nominators (active vs waiting).
- Network total staked percentage.
- Validator status, identity and Live Points
- Total nominators, Total stake, Self stake, Bonded, Unbonding, Unlocked. Display payee.
- Active vs Next commission. Current and Queued session keys;
- Validate and display proxy type for each stash.
- Autocomplete, select or filter commands (extrinsics) based on proxy type context.
- Support for `/bond`, `/bond_extra`, `/unbond`, `/rebond`, `/withdraw_unbonded`, `/validate`, `/chill`, `/set_keys`, `/purge_keys`, `/set_keys_async`, `/purge_keys_async`.
- Verify and sign call_data. Display and log transaction progress.
- Update metadata polkadot/2000007
- Update metadata asset-hub-polkadot/2000007
- Update metadata people-polkadot/2000007
- Update metadata kusama/2001000
- Update metadata asset-hub-kusama/2001000
- Update metadata people-kusama/2001000
- Update metadata westend/1022000
- Update metadata asset-hub-westend/1022000
- Update metadata people-kusama/1022000
