use crate::error::Error;
use openssh::{Session, SessionBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;

/// Default SSH PORT
fn default_ssh_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshConfig {
    pub host: String,
    pub user: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    #[serde(default)]
    pub identity: Option<String>, // path to private key, None = use SSH agent
}

impl SshConfig {
    pub fn host(&self, masked: bool) -> String {
        if masked {
            return "X.X.X.X".to_string();
        }
        self.host.to_string()
    }
}

pub enum NodeAccess {
    Local,
    Ssh(SshConfig),
}

impl NodeAccess {
    pub fn from_ssh_config(ssh: Option<&SshConfig>) -> Self {
        match ssh {
            Some(ssh) => Self::Ssh(ssh.clone()),
            None => Self::Local,
        }
    }

    pub async fn execute_rpc(&self, payload: &str, url: &str) -> Result<Vec<u8>, Error> {
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
                    .map_err(|e| Error::LocalExecution(format!("curl failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(Error::LocalExecution(format!(
                        "curl exited with error: {}",
                        stderr
                    )));
                }
                Ok(output.stdout)
            }
            Self::Ssh(config) => {
                let session = open_ssh_session(config).await?;

                let output_result = session
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
                    .map_err(|e| Error::RemoteExecution(format!("Remote curl failed: {}", e)));

                let result = match output_result {
                    Err(e) => Err(e),
                    Ok(output) if !output.status.success() => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Err(Error::RemoteExecution(format!(
                            "curl exited with error: {}",
                            stderr
                        )))
                    }
                    Ok(output) => Ok(output.stdout),
                };

                session.close().await.ok();
                result
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
                    .map_err(|e| Error::LocalExecution(format!("Shell failed: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(Error::LocalExecution(format!(
                        "shell exited with error: {}",
                        stderr
                    )));
                }

                let stdout = String::from_utf8_lossy(&output.stdout).to_string();

                Ok(stdout)
            }
            Self::Ssh(config) => {
                let session = open_ssh_session(config).await?;

                let output_result = session
                    .command("sh")
                    .arg("-c")
                    .arg(run)
                    .output()
                    .await
                    .map_err(|e| Error::RemoteExecution(format!("Remote shell failed: {}", e)));

                let result = match output_result {
                    Err(e) => Err(e),
                    Ok(output) if !output.status.success() => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        Err(Error::RemoteExecution(format!(
                            "curl exited with error: {}",
                            stderr
                        )))
                    }
                    Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
                };

                session.close().await.ok();

                result
            }
        }
    }
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
