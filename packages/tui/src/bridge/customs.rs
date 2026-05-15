use openssh::{Session, SessionBuilder};
use std::str::FromStr;
use std::time::Duration;
use suno_config::SshConfig;
use suno_error::Error;
use suno_primitives::{
    session::{Keys, Proof},
    Validator,
};
use tokio::process::Command;

pub async fn process(run: &str, validator: &Validator) -> Result<String, Error> {
    let run = run.replace("{stash}", &validator.key().stash().to_string());

    if let Some(ssh) = &validator.ssh {
        return process_via_ssh(ssh, &run).await;
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(&run)
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

pub async fn process_via_ssh(ssh: &SshConfig, run: &str) -> Result<String, Error> {
    let session = open_ssh_session(ssh).await?;

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

pub async fn rotate_keys(validator: &Validator) -> Result<(Keys, Proof), Error> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "author_rotateKeysWithOwner",
        "params": [validator.account.public_key()],
        "id": 1
    })
    .to_string();

    if let Some(ssh) = &validator.ssh {
        return rotate_keys_via_ssh(ssh, &payload, &validator.host_rpc.http_url()).await;
    }

    rotate_keys_via_http(&payload, &validator.host_rpc.http_url()).await
}

pub async fn rotate_keys_via_ssh(
    ssh: &SshConfig,
    payload: &str,
    url: &str,
) -> Result<(Keys, Proof), Error> {
    let session = open_ssh_session(ssh).await?;

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

    let result = parse_rotate_keys_response(&output.stdout)?;

    session.close().await.ok();

    Ok(result)
}

pub async fn rotate_keys_via_http(payload: &str, url: &str) -> Result<(Keys, Proof), Error> {
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

    parse_rotate_keys_response(&output.stdout)
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
