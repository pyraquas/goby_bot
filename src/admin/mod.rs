//! Administration commands and the guild setup they perform.

pub mod commands;
pub mod init;
pub mod role_select;

/// Role gating the Magic: the Gathering feature.
pub const MTG_ROLE: &str = "Politicien des cartes";
/// Category holding the Magic: the Gathering channels.
pub const MTG_CATEGORY: &str = "Magic: le rassemblement";
/// Default text channel of the Magic: the Gathering category.
pub const MTG_CHANNEL: &str = "magic-general";

/// Role gating the Archipelago feature.
pub const ARCHI_ROLE: &str = "Explorateur des îles";
/// Category holding the Archipelago channels.
pub const ARCHI_CATEGORY: &str = "Archipelago";
/// Default text channel of the Archipelago category.
pub const ARCHI_CHANNEL: &str = "archipelago-general";

/// Read only channel where members pick their roles.
pub const JOB_CHANNEL: &str = "abaye-des-vocations";
