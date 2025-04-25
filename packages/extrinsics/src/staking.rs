use subxt::{
    dynamic::{tx, Value},
    tx::DynamicPayload,
};

pub fn chill() -> DynamicPayload {
    tx("Staking", "chill", vec![] as Vec<Value>)
}
