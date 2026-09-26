use ratatui::widgets::TableState;
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
use suno_primitives::{
    collator::{Collator, CollatorStatus},
    AccountDisplay,
};

#[derive(Debug, Default)]
pub struct CollatorsList {
    pub collators: Vec<Collator>,
    pub table_state: TableState,
    active: bool,
}

impl CollatorsList {
    pub fn on_init(&mut self) {
        let chains = CONFIG.chains();
        for chain in chains.iter() {
            for (chain_name, chain_config) in chain {
                for collator in &chain_config.collators {
                    match collator {
                        NodeConfig::Address(stash) => {
                            self.collators.push(Collator::new(*chain_name, *stash));
                        }
                        NodeConfig::Detailed { stash, .. } => {
                            self.collators.push(Collator::new(*chain_name, *stash));
                        }
                    }
                }
            }
        }
        if !self.collators.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    /// Returns an iterator of collators in display order
    pub fn collators_iter(&self) -> impl Iterator<Item = &Collator> {
        self.collators.iter()
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn get_selected(&self) -> Option<Collator> {
        self.table_state
            .selected()
            .and_then(|i| self.collators.get(i).cloned())
    }

    pub fn move_down(&mut self) -> Option<Collator> {
        if let Some(selected) = self.table_state.selected() {
            if selected == self.collators.len() - 1 {
                self.table_state.select_first();
            } else {
                self.table_state.scroll_down_by(1);
            }
            self.table_state
                .selected()
                .and_then(|i| self.collators.get(i).cloned())
        } else {
            None
        }
    }

    pub fn move_up(&mut self) -> Option<Collator> {
        if let Some(selected) = self.table_state.selected() {
            if selected == 0 {
                let i = self.collators.len() - 1;
                self.table_state.select(Some(i));
            } else {
                self.table_state.scroll_up_by(1);
            }
            self.table_state
                .selected()
                .and_then(|i| self.collators.get(i).cloned())
        } else {
            None
        }
    }

    /// Marks the collators of `runtime` found in the current Aura authority set.
    pub fn update_aura_authorities(&mut self, runtime: SupportedRuntime, authorities: &[[u8; 32]]) {
        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime {
                let stash_bytes: [u8; 32] = *collator.stash().as_ref();
                let status = if authorities.contains(&stash_bytes) {
                    CollatorStatus::Authority
                } else {
                    CollatorStatus::Unknown
                };
                collator.set_status(status);
            }
        }
    }
}
