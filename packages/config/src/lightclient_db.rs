use crate::config::write_private_atomic;
use crate::error::Error;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use subxt::lightclient::LightClientRpc;
use tracing::{debug, info, warn};

/// Databases older than this are discarded.
const MAX_AGE: Duration = Duration::from_secs(3 * 24 * 60 * 60);
/// Upper bound for the exported database size.
const MAX_SIZE_BYTES: u64 = 10_000_000;
/// Delay before the first save, giving the light client time to sync.
const FIRST_SAVE_DELAY: Duration = Duration::from_secs(2 * 60);
/// Interval between periodic saves.
const SAVE_INTERVAL: Duration = Duration::from_secs(10 * 60);

static REGISTERED: Mutex<Vec<(PathBuf, LightClientRpc)>> = Mutex::new(Vec::new());

/// Loads the saved database, or an empty string (a normal sync) if missing or not trusted.
pub fn load(path: &Path) -> String {
    match try_load(path) {
        Ok(Some(db)) => {
            info!("Light client database loaded from {}", path.display());
            db
        }
        Ok(None) => String::new(),
        Err(reason) => {
            warn!(
                "Light client database {} ignored: {}",
                path.display(),
                reason
            );
            String::new()
        }
    }
}

/// Reads the database, if it exists and is trusted: a regular file owned by the current
/// user, not writable by others, and not older than `MAX_AGE`.
fn try_load(path: &Path) -> Result<Option<String>, Error> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if !meta.is_file() {
        return Err(Error::InvalidPath(path.display().to_string()));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if meta.uid() != unsafe { libc::geteuid() } {
            return Err(Error::Other("not owned by the current user".into()));
        }
        if meta.permissions().mode() & 0o022 != 0 {
            return Err(Error::Other("writable by group or others".into()));
        }
    }
    let age = meta
        .modified()
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .ok_or_else(|| Error::Other("invalid modification time".into()))?;
    if age > MAX_AGE {
        return Err(Error::Other("too old".into()));
    }
    Ok(Some(fs::read_to_string(path)?))
}

async fn save(path: &Path, rpc: &LightClientRpc) {
    match rpc.database_content(Some(MAX_SIZE_BYTES)).await {
        Ok(db) if db.is_empty() => debug!("Light client database is empty, not saved"),
        Ok(db) => match write_private_atomic(path, db.as_bytes()) {
            Ok(()) => debug!("Light client database saved to {}", path.display()),
            Err(e) => warn!("Light client database not saved: {e}"),
        },
        Err(e) => debug!("Light client database not exported: {e}"),
    }
}

/// Registers a relay chain light client: saves its database periodically and on `save_all`.
pub fn register(path: PathBuf, rpc: LightClientRpc) {
    REGISTERED.lock().unwrap().push((path.clone(), rpc.clone()));
    tokio::spawn(async move {
        tokio::time::sleep(FIRST_SAVE_DELAY).await;
        loop {
            save(&path, &rpc).await;
            tokio::time::sleep(SAVE_INTERVAL).await;
        }
    });
}

/// Saves the databases of all registered light clients, to be called on exit.
pub async fn save_all() {
    let entries: Vec<_> = REGISTERED.lock().unwrap().clone();
    for (path, rpc) in entries {
        let _ = tokio::time::timeout(Duration::from_secs(3), save(&path, &rpc)).await;
    }
}
