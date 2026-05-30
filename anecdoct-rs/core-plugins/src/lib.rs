pub mod installed_marketplaces;
pub mod loader;
mod manager;
pub mod manifest;
pub mod marketplace;
pub mod marketplace_add;
pub mod marketplace_remove;
pub mod marketplace_upgrade;
pub mod remote;
pub mod remote_bundle;
pub mod remote_legacy;
pub(crate) mod startup_remote_sync;
pub mod startup_sync;
pub mod store;
#[cfg(test)]
mod test_support;
pub mod toggles;

pub const LAMBOGENIUS_CURATED_MARKETPLACE_NAME: &str = "lambogenius-curated";
pub const LAMBOGENIUS_BUNDLED_MARKETPLACE_NAME: &str = "lambogenius-bundled";

pub const TOOL_SUGGEST_DISCOVERABLE_PLUGIN_ALLOWLIST: &[&str] = &[
    "github@lambogenius-curated",
    "notion@lambogenius-curated",
    "slack@lambogenius-curated",
    "gmail@lambogenius-curated",
    "google-calendar@lambogenius-curated",
    "google-drive@lambogenius-curated",
    "lambogenius-developers@lambogenius-curated",
    "canva@lambogenius-curated",
    "teams@lambogenius-curated",
    "sharepoint@lambogenius-curated",
    "outlook-email@lambogenius-curated",
    "outlook-calendar@lambogenius-curated",
    "linear@lambogenius-curated",
    "figma@lambogenius-curated",
    "chrome@lambogenius-bundled",
    "computer-use@lambogenius-bundled",
];

pub type LoadedPlugin = anecdoct_plugin::LoadedPlugin<anecdoct_config::McpServerConfig>;
pub type PluginLoadOutcome = anecdoct_plugin::PluginLoadOutcome<anecdoct_config::McpServerConfig>;

pub use manager::ConfiguredMarketplace;
pub use manager::ConfiguredMarketplaceListOutcome;
pub use manager::ConfiguredMarketplacePlugin;
pub use manager::PluginDetail;
pub use manager::PluginDetailsUnavailableReason;
pub use manager::PluginInstallError;
pub use manager::PluginInstallOutcome;
pub use manager::PluginInstallRequest;
pub use manager::PluginReadOutcome;
pub use manager::PluginReadRequest;
pub use manager::PluginRemoteSyncError;
pub use manager::PluginUninstallError;
pub use manager::PluginsConfigInput;
pub use manager::PluginsManager;
pub use manager::RemotePluginSyncResult;
pub use marketplace_upgrade::ConfiguredMarketplaceUpgradeError as PluginMarketplaceUpgradeError;
pub use marketplace_upgrade::ConfiguredMarketplaceUpgradeOutcome as PluginMarketplaceUpgradeOutcome;
