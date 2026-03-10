use crate::node_runtime::runtime_types::{
    polkadot_primitives::v8::{
        assignment_app::Public as AssignmentPublic, validator_app::Public as ValidatorPublic,
    },
    polkadot_runtime::SessionKeys,
    sp_authority_discovery::app::Public as AuthorityDiscoveryPublic,
    sp_consensus_babe::app::Public as BabePublic,
    sp_consensus_beefy::ecdsa_crypto::Public as BeefyPublic,
    sp_consensus_grandpa::app::Public as GrandpaPublic,
};
use suno_primitives::session::Keys;

/// Helper function to map SessionKeys to Keys
pub fn map_keys_from_session_keys(session_keys: &SessionKeys) -> Keys {
    let GrandpaPublic(grandpa) = session_keys.grandpa;
    let BabePublic(babe) = session_keys.babe;
    let ValidatorPublic(para) = session_keys.para_validator;
    let AssignmentPublic(assi) = session_keys.para_assignment;
    let AuthorityDiscoveryPublic(auth) = session_keys.authority_discovery;
    let BeefyPublic(beef) = session_keys.beefy;
    Keys::new(grandpa, babe, para, assi, auth, beef)
}
