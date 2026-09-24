//! Portaki appliances module — device guides for the guest booklet.

mod commands;
mod content;
mod entities;
mod guest;
mod host;
mod ids;
mod queries;
mod store;

pub use commands::{
    delete_appliance, reorder_appliances, replace_devices, save_appliance, save_safety_notice,
    DeleteApplianceArgs, ReorderAppliancesArgs, ReplaceDeviceSlot, ReplaceDevicesArgs,
    SaveApplianceArgs, SaveSafetyNoticeArgs,
};
pub use content::{Appliance, ApplianceStatus, AppliancesPayload, MAX_APPLIANCES, MAX_FEATURED};
pub use entities::AppliancesContent;
pub use guest::{render_explore_detail, render_explore_item, render_home_card};
pub use host::render_host_main;
pub use queries::{get_content, AppliancesContentView, GetContentArgs};
pub use store::reset_test_store;

/// Test-only: write raw JSON into the content slot (legacy or v2).
#[cfg(any(test, debug_assertions))]
pub fn store_save_legacy_for_tests(content_fr: String) -> portaki_sdk::prelude::Result<()> {
    let _ = store::save_content_row(content_fr, String::new())?;
    Ok(())
}

portaki_sdk::portaki_module!(
    id = "appliances",
    display_name_key = "module.displayName",
    description_key = "module.catalogDescription",
    author = "Portaki",
    author_url = "https://portaki.app",
    module_type = "official",
    icon = "plug",
    maturity = "stable",
    sort_order = 120,
);

#[portaki_sdk::capability(required, id = "core.storage")]
pub const STORAGE: &str = "core.storage";
