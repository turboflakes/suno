use openssh::{Session, SessionBuilder};
use std::str::FromStr;
use std::time::Duration;
use suno_config::SshConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::{
    session::{Keys, KeysError, Proof},
    Validator,
};
use tokio::process::Command;

pub enum NodeAccess {
    Local,
    Ssh(SshConfig),
}

impl NodeAccess {
    pub fn from_validator(validator: &Validator) -> Self {
        match &validator.ssh {
            Some(ssh) => Self::Ssh(ssh.clone()),
            None => Self::Local,
        }
    }

    async fn execute_rpc(&self, payload: &str, url: &str) -> Result<Vec<u8>, Error> {
        match &self {
            Self::Local => {
                let output = Command::new("curl")
                    .arg("-s")
                    .arg("-X")
                    .arg("POST")
                    .arg("-H")
                    .arg("Content-Type: application/json")
                    .arg("-d")
                    .arg(payload)
                    .arg(url)
                    .output()
                    .await
                    .map_err(|e| Error::Other(format!("curl failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(Error::Other(format!("curl exited with error: {}", stderr)));
                }
                Ok(output.stdout)
            }
            Self::Ssh(config) => {
                let session = open_ssh_session(config).await?;

                let output = session
                    .command("curl")
                    .arg("-s")
                    .arg("-X")
                    .arg("POST")
                    .arg("-H")
                    .arg("Content-Type: application/json")
                    .arg("-d")
                    .arg(payload)
                    .arg(url)
                    .output()
                    .await
                    .map_err(|e| Error::Other(format!("Remote curl failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(Error::Other(format!("curl exited with error: {}", stderr)));
                }

                session.close().await.ok();

                Ok(output.stdout)
            }
        }
    }

    pub async fn execute_shell(&self, run: &str) -> Result<String, Error> {
        match &self {
            Self::Local => {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(run)
                    .output()
                    .await
                    .map_err(|e| Error::Other(format!("Shell failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(Error::Other(format!("shell exited with error: {}", stderr)));
                }

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                Ok(stdout)
            }
            Self::Ssh(config) => {
                let session = open_ssh_session(config).await?;

                let output = session
                    .command("sh")
                    .arg("-c")
                    .arg(run)
                    .output()
                    .await
                    .map_err(|e| Error::Other(format!("Remote shell failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(Error::Other(format!("shell exited with error: {}", stderr)));
                }

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                session.close().await.ok();

                Ok(stdout)
            }
        }
    }
}

pub async fn rotate_keys(validator: &Validator) -> Result<(Keys, Proof), Error> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "author_rotateKeysWithOwner",
        "params": [validator.account.public_key()],
        "id": 1
    })
    .to_string();
    let stdout = NodeAccess::from_validator(validator)
        .execute_rpc(&payload, &validator.host_rpc.http_url())
        .await?;
    parse_rotate_keys_response(&stdout)
}

fn parse_rotate_keys_response(stdout: &[u8]) -> Result<(Keys, Proof), Error> {
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

    let keys = Keys::from_str(keys_hex).map_err(|e| Error::Other(e.to_string()))?;
    let proof = Proof::from_str(proof_hex).map_err(|e| Error::Other(e.to_string()))?;

    Ok((keys, proof))
}

pub async fn has_keys(validator: &Validator) -> Result<bool, Error> {
    let keys = validator
        .next_keys
        .as_ref()
        .ok_or(KeysError::NotSet)
        .boxed()?;

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "author_hasSessionKeys",
        "params": [keys.to_string()],
        "id": 1
    })
    .to_string();
    let stdout = NodeAccess::from_validator(validator)
        .execute_rpc(&payload, &validator.host_rpc.http_url())
        .await?;
    parse_has_keys_response(&stdout)
}

pub async fn has_queued_keys(validator: &Validator) -> Result<bool, Error> {
    let keys = validator
        .queued_keys
        .as_ref()
        .ok_or(KeysError::NotSet)
        .boxed()?;

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "author_hasSessionKeys",
        "params": [keys.to_string()],
        "id": 1
    })
    .to_string();
    let stdout = NodeAccess::from_validator(validator)
        .execute_rpc(&payload, &validator.host_rpc.http_url())
        .await?;
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

async fn open_ssh_session(ssh: &SshConfig) -> Result<Session, Error> {
    let mut builder = SessionBuilder::default();
    builder
        .user(ssh.user.clone())
        .port(ssh.port)
        .connect_timeout(Duration::from_secs(15))
        .known_hosts_check(openssh::KnownHosts::Strict);

    if let Some(identity) = &ssh.identity {
        builder.keyfile(shellexpand::tilde(identity).as_ref());
    }

    builder.connect(&ssh.host).await.map_err(|e| {
        if e.to_string().to_lowercase().contains("authentication") {
            Error::Other(format!(
                "SSH authentication failed for {}. Run: ssh-add {}",
                ssh.host,
                ssh.identity.as_deref().unwrap_or("~/.ssh/id_ed25519")
            ))
        } else {
            Error::Other(e.to_string())
        }
    })
}
