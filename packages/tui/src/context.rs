use crate::call::Call;
use subxt::OnlineClient;
use suno_config::{CustomConfig, SupportedRuntime};
use suno_primitives::Validator;

#[derive(Debug, Clone)]
pub struct Context {
    pub api: OnlineClient<CustomConfig>,
    pub runtime: SupportedRuntime,
    pub validator: Validator,
    pub call: Option<Call>,
}

impl Context {
    pub fn new(
        api: OnlineClient<CustomConfig>,
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
