use subxt::{
    dynamic::{tx, Value},
    tx::DynamicPayload,
    utils::AccountId32,
};

pub fn proxy(proxied_account: AccountId32, payload: DynamicPayload) -> DynamicPayload {
    let proxied_account_bytes: &[u8] = proxied_account.as_ref();
    tx(
        "Proxy",
        "proxy",
        vec![
            (
                "real",
                Value::unnamed_variant("Id", vec![Value::from_bytes(proxied_account_bytes)]),
            ),
            (
                "force_proxy_type",
                Value::unnamed_variant("Some", vec![Value::unnamed_variant("Any", vec![])]),
            ),
            ("call", payload.into_value()),
        ],
    )
}
