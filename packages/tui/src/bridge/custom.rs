use std::str::FromStr;
use suno_config::NodeAccess;
use suno_error::{Error, ResultExt};
use suno_primitives::{
    session::{KeysError, Proof},
    NodeHost,
};

pub async fn rotate_keys<N: NodeHost>(node: &N) -> Result<(N::SessionKeys, Proof), Error> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "author_rotateKeysWithOwner",
        "params": [node.public_key()],
        "id": 1
    })
    .to_string();
    let stdout = NodeAccess::from_ssh_config(node.ssh())
        .execute_rpc(&payload, &node.host_rpc().http_url())
        .await
        .boxed()?;
    parse_rotate_keys_response::<N::SessionKeys>(&stdout)
}

fn parse_rotate_keys_response<K: FromStr<Err = KeysError>>(
    stdout: &[u8],
) -> Result<(K, Proof), Error> {
    let response: serde_json::Value = serde_json::from_slice(stdout)
        .map_err(|e| Error::Other(format!("Invalid JSON from curl: {}", e)))?;

    if let Some(err) = response.get("error") {
        return Err(Error::Other(format!("RPC error: {}", err)));
    }

    let result = response.get("result").ok_or(Error::Other(
        "Missing 'result' field in RPC response".into(),
    ))?;

    let keys_hex = result
        .get("keys")
        .and_then(|v| v.as_str())
        .ok_or(Error::Other("Missing 'keys' field in RPC response".into()))?;

    let proof_hex = result
        .get("proof")
        .and_then(|v| v.as_str())
        .ok_or(Error::Other("Missing 'proof' field in RPC response".into()))?;

    let keys = K::from_str(keys_hex).map_err(|e| Error::Other(e.to_string()))?;
    let proof = Proof::from_str(proof_hex).map_err(|e| Error::Other(e.to_string()))?;

    Ok((keys, proof))
}

pub async fn has_keys<N: NodeHost>(node: &N) -> Result<bool, Error> {
    let keys = node.next_session_keys().ok_or(KeysError::NotSet).boxed()?;
    has_session_keys(node, keys).await
}

pub async fn has_queued_keys<N: NodeHost>(node: &N) -> Result<bool, Error> {
    let keys = node
        .queued_session_keys()
        .ok_or(KeysError::NotSet)
        .boxed()?;
    has_session_keys(node, keys).await
}

async fn has_session_keys<N: NodeHost>(node: &N, keys: N::SessionKeys) -> Result<bool, Error> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "author_hasSessionKeys",
        "params": [keys.to_string()],
        "id": 1
    })
    .to_string();
    let stdout = NodeAccess::from_ssh_config(node.ssh())
        .execute_rpc(&payload, &node.host_rpc().http_url())
        .await
        .boxed()?;
    parse_has_keys_response(&stdout)
}

fn parse_has_keys_response(stdout: &[u8]) -> Result<bool, Error> {
    let response: serde_json::Value = serde_json::from_slice(stdout)
        .map_err(|e| Error::Other(format!("Invalid JSON from curl: {}", e)))?;

    if let Some(err) = response.get("error") {
        return Err(Error::Other(format!("RPC error: {}", err)));
    }

    let result = response
        .get("result")
        .ok_or(Error::Other(
            "Missing 'result' field in RPC response".into(),
        ))?
        .as_bool()
        .ok_or(Error::Other("'result' field is not a boolean".into()))?;

    Ok(result)
}
