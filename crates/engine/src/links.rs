//! Community and project links (shown in Help, About, the CLI and MCP). A rebranded build
//! ([`crate::brand`]) points them at its own pages: the ArtCraft Discord and website are only the
//! release's (docs/brand/LICENSE-brand.txt).

/// Where people get help: the ArtCraft Discord in the release, the build's support page otherwise.
pub const DISCORD: &str = crate::brand::SUPPORT;
pub const WEBSITE: &str = if crate::brand::REBRANDED { crate::brand::APP_PAGE } else { "https://getartcraft.com/" };
pub const APPS: &str = if crate::brand::REBRANDED { crate::brand::APP_PAGE } else { "https://getartcraft.com/apps" };
pub const APP_PAGE: &str = crate::brand::APP_PAGE;
pub const GITHUB: &str = crate::brand::GITHUB;
