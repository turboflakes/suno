use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Built-in call methods derived from `Call`'s Display implementation.
/// Must be kept in sync with `suno_primitives::call::Call`.
const BUILTIN_CALL_NAMES: &[&str] = &[
    "bond",
    "bond_extra",
    "unbond",
    "rebond",
    "withdraw_unbonded",
    "set_payee",
    "validate",
    "chill",
    "set_keys",
    "purge_keys",
    "set_keys_async",
    "purge_keys_async",
    "rotate_and_set_keys", // CustomCalls::RotateAndSetKeys
    "has_keys",            // CustomCalls::HasKeys
    "has_queued_keys",     // CustomCalls::HasQueuedKeys
];

/// Placeholders resolved automatically from the validator context, not from user input.
const BUILTIN_PLACEHOLDERS: &[&str] = &["stash"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomCommand {
    pub name: String,
    #[serde(flatten)]
    pub kind: CommandKind,
}

impl std::fmt::Display for CustomCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            CommandKind::Shell { run, .. } => write!(f, "Run '{}'", run.to_string()),
            CommandKind::Uses(custom) => write!(f, "{}", custom.description()),
        }
    }
}

impl CustomCommand {
    pub fn cmd(&self) -> String {
        match &self.kind {
            CommandKind::Shell { cmd: None, .. } => {
                self.name.trim().to_lowercase().replace(" ", "_")
            }
            _ => self.kind.to_string(),
        }
    }

    pub fn is_shell(&self) -> bool {
        matches!(self.kind, CommandKind::Shell { .. })
    }

    pub fn is_super(&self) -> bool {
        matches!(self.kind, CommandKind::Uses(..))
    }

    /// Command name without `{...}` placeholders.
    /// e.g. "/echo {msg}" -> "msg"
    pub fn base_cmd(&self) -> String {
        self.cmd()
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Display label with `{arg}` placeholders shown as `<arg>` for the UI hint
    /// e.g. "/echo {msg}" -> "/echo <msg>"
    pub fn placeholder(&self) -> String {
        self.cmd().replace("{", "<").replace("}", ">")
    }

    /// Unique placeholder names in `run`, excluding built-ins like `{stash}`.
    /// run = "echo {msg}" -> ["msg"]
    pub fn args(&self) -> Vec<String> {
        let CommandKind::Shell { run, .. } = &self.kind else {
            return vec![];
        };
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        let mut s = run.as_str();
        while let Some(open) = s.find('{') {
            s = &s[open + 1..];
            if let Some(close) = s.find('}') {
                let name = &s[..close];
                if !name.is_empty()
                    && !BUILTIN_PLACEHOLDERS.contains(&name)
                    && seen.insert(name.to_string())
                {
                    result.push(name.to_string());
                }
                s = &s[close + 1..];
            }
        }
        result
    }

    /// Returns a clone with `{name}` placeholders substituted positionally.
    /// Also substitutes in `cmd` so the display label reflects the resolved value.
    pub fn with_args(&self, values: &[&str]) -> Self {
        let mut cloned = self.clone();
        if let CommandKind::Shell { run, cmd } = &mut cloned.kind {
            for (name, value) in self.args().iter().zip(values) {
                let placeholder = format!("{{{}}}", name);
                // NOTE: escape the user value before substituting into the run template.
                let escaped = format!("'{}'", value.replace('\'', "'\\''"));
                *run = run.replace(&placeholder, &escaped);
                if let Some(c) = cmd.as_mut() {
                    *c = c.replace(&placeholder, value);
                }
            }
        }
        cloned
    }

    pub fn validate(&self) -> Result<(), Error> {
        if let CommandKind::Shell { .. } = &self.kind {
            let cmd = self.base_cmd();
            if BUILTIN_CALL_NAMES.contains(&cmd.as_str()) {
                return Err(Error::InvalidCommand(cmd));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandKind {
    /// Built-in command: "calls/rotate_and_set_keys"
    Uses(CustomCalls),
    /// Shell command: "echo test"
    #[serde(untagged)]
    Shell {
        #[serde(default)]
        cmd: Option<String>, // shown in UI; falls back to `name` if absent
        run: String, // command to be executed: "echo test", "systemctl restart ..."
    },
}

impl std::fmt::Display for CommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shell { cmd, .. } => {
                let display = cmd
                    .as_deref()
                    .and_then(|s| s.strip_prefix('/'))
                    .unwrap_or("ND");
                write!(f, "{}", display)
            }
            Self::Uses(call) => write!(f, "{}", call),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomCalls {
    #[serde(rename = "calls/rotate_and_set_keys")]
    RotateAndSetKeys,
    #[serde(rename = "calls/has_keys")]
    HasKeys,
    #[serde(rename = "calls/has_queued_keys")]
    HasQueuedKeys,
}

impl CustomCalls {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RotateAndSetKeys => "rotate_and_set_keys",
            Self::HasKeys => "has_keys",
            Self::HasQueuedKeys => "has_queued_keys",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::RotateAndSetKeys => "Atomically rotate and set session keys",
            Self::HasKeys => "Check whether the host has the current session keys",
            Self::HasQueuedKeys => "Check whether the host has queued session keys",
        }
    }
}

impl std::fmt::Display for CustomCalls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
