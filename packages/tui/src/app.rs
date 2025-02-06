use crate::config::{SupportedRuntime, CONFIG};
use crate::widgets::chains::ChainsListWidget;
use log::{error, info, warn};
use std::{collections::HashMap, time::Duration};
use subxt::{
    backend::rpc::reconnecting_rpc_client::{ExponentialBackoff, RpcClient},
    utils::validate_url_is_secure,
    OnlineClient, SubstrateConfig,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Holds the API clients for each supported runtime.
    pub chains: ChainsListWidget,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: true,
            chains: ChainsListWidget::default(),
        }
    }

    pub async fn init(&mut self) {
        self.chains.run().await;
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    // pub fn increment_counter(&mut self) {
    //     if let Some(res) = self.counter.checked_add(1) {
    //         self.counter = res;
    //     }
    // }

    // pub fn decrement_counter(&mut self) {
    //     if let Some(res) = self.counter.checked_sub(1) {
    //         self.counter = res;
    //     }
    // }
}
