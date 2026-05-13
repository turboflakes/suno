use openssh::{ForwardType, Session, SessionBuilder};
use std::str::FromStr;
use std::{net::TcpListener, time::Duration};
use subxt_rpcs::{client::RpcParams, RpcClient};
use suno_config::{Host, SshConfig};
use suno_error::Error;
use suno_primitives::{session::Keys, Validator};

pub async fn rotate_keys_via_ssh(validator: Validator) -> Result<Keys, Error> {
    let ssh = validator
        .ssh
        .ok_or_else(|| Error::Other("SSH not configured".into()))?;

    let (tunnel, session) = open_local_ssh_tunnel(ssh, validator.host_rpc).await?;

    let client = RpcClient::from_insecure_url(&tunnel.http_url())
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    let keys = rotate_keys_with_owner(&client, validator.account.public_key()).await?;

    session.close().await.ok();

    Ok(keys)
}

pub async fn rotate_keys_with_owner(client: &RpcClient, owner: String) -> Result<Keys, Error> {
    let mut params = RpcParams::new();
    params
        .push(owner)
        .map_err(|e| Error::Other(e.to_string()))?;

    let result: serde_json::Value = client
        .request("author_rotateKeysWithOwner", params)
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    let keys_hex = result
        .as_str()
        .ok_or(Error::Other("Invalid RPC response".into()))?;

    Keys::from_str(keys_hex).map_err(|e| Error::Other(e.to_string()))
}

async fn open_local_ssh_tunnel(ssh: SshConfig, target: Host) -> Result<(Host, Session), Error> {
    let mut builder = SessionBuilder::default();
    builder
        .user(ssh.user.clone())
        .port(ssh.port)
        .connect_timeout(Duration::from_secs(6))
        .known_hosts_check(openssh::KnownHosts::Strict);

    if let Some(identity) = &ssh.identity {
        builder.keyfile(shellexpand::tilde(identity).as_ref());
    }

    let session = builder.connect(&ssh.host).await.map_err(|e| {
        if e.to_string().to_lowercase().contains("authentication") {
            Error::Other(format!(
                "SSH authentication failed for {}. Run: ssh-add {}",
                ssh.host,
                ssh.identity.as_deref().unwrap_or("~/.ssh/id_ed25519")
            ))
        } else {
            Error::Other(e.to_string())
        }
    })?;

    let local = Host::new_with_port(get_free_port());
    session
        .request_port_forward(ForwardType::Local, local.as_tuple(), target.as_tuple())
        .await
        .map_err(|e| Error::Other(e.to_string()))?;

    Ok((local, session))
}

fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to an available port");

    listener
        .local_addr()
        .expect("Failed to get local address")
        .port()
}
