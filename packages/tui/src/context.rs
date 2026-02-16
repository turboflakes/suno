use crate::call::Call;
use subxt::{OnlineClient, SubstrateConfig};
use suno_config::SupportedRuntime;
use suno_primitives::Validator;

#[derive(Debug, Clone)]
pub struct Context {
    pub api: OnlineClient<SubstrateConfig>,
    pub runtime: SupportedRuntime,
    pub validator: Validator,
    pub call: Option<Call>,
}

impl Context {
    pub fn new(
        api: OnlineClient<SubstrateConfig>,
        runtime: SupportedRuntime,
        validator: Validator,
        call: Option<Call>,
    ) -> Self {
        Self {
            api,
            runtime,
            validator,
            call,
        }
    }
}
