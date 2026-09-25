//! Module commands — nearby cache refresh. `updateConfig` is the platform's
//! (`#[portaki_sdk::config]`).

use portaki_sdk::prelude::*;

use crate::nearby::invalidate_nearby_cache;

#[portaki_sdk::command(name = "refreshNearby")]
pub fn refresh_nearby(_ctx: Context) -> Result<()> {
    invalidate_nearby_cache()
}
