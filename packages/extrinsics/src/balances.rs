use subxt::{
    dynamic::{tx, Value},
    tx::DynamicPayload,
    utils::AccountId32,
};

pub fn transfer_keep_alive(dest: AccountId32, value: u128) -> DynamicPayload {
    let dest_bytes: &[u8] = dest.as_ref();

    tx(
        "Balances",
        "transfer_keep_alive",
        vec![
            (
                "dest",
                Value::unnamed_variant("Id", vec![Value::from_bytes(dest_bytes)]),
            ),
            ("value", Value::u128(value)),
        ],
    )
}
