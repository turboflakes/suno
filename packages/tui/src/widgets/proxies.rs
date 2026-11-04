use ratatui::widgets::TableState;
use suno_primitives::proxy::ProxyState;
use suno_theme::Theme;

/// The proxy accounts of every chain, one row each.
#[derive(Debug, Default)]
pub struct ProxiesList {
    proxies: Vec<ProxyState>,
    pub(crate) table_state: TableState,
    is_active: bool,
    theme: Theme,
}

impl ProxiesList {
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    /// Replaces the proxy accounts listed, keeping the one selected when it is still there.
    pub fn load(&mut self, proxies: Vec<ProxyState>) {
        let selected = self.get_selected();
        let index = self.table_state.selected().unwrap_or(0);
        self.proxies = proxies;

        let position = selected.and_then(|s| {
            self.proxies
                .iter()
                .position(|p| p.runtime == s.runtime && p.account == s.account)
        });
        let len = self.proxies.len();
        self.table_state
            .select(position.or_else(|| (len > 0).then(|| index.min(len - 1))));
    }

    pub fn proxies_iter(&self) -> impl Iterator<Item = &ProxyState> {
        self.proxies.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    pub fn get_selected(&self) -> Option<ProxyState> {
        self.proxies.get(self.table_state.selected()?).cloned()
    }

    pub fn move_down(&mut self) {
        let len = self.proxies.len();
        if len == 0 {
            return;
        }
        let selected = self.table_state.selected().unwrap_or(0).min(len - 1);
        self.table_state.select(Some((selected + 1) % len));
    }

    pub fn move_up(&mut self) {
        let len = self.proxies.len();
        if len == 0 {
            return;
        }
        let selected = self.table_state.selected().unwrap_or(0).min(len - 1);
        self.table_state.select(Some((selected + len - 1) % len));
    }
}
