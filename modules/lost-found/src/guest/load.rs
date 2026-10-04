//! Load stay reports for guest surfaces.

use chrono::Duration;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;
use crate::entities::LostFoundReport;
use crate::storage;

pub struct GuestData {
    pub reports: Vec<LostFoundReport>,
    /// Le délai de signalement est passé : on ne propose plus le formulaire (§10).
    pub window_closed: bool,
    /// Les options de restitution que l'hôte propose, `ship` · `pickup` · `donate`.
    pub return_options: Vec<&'static str>,
    /// Le renvoi est proposé, et c'est le voyageur qui en paie les frais.
    pub shipping_paid_by_guest: bool,
    /// Le prénom de l'hôte, pour « Claire cherche ». Vide quand la plateforme ne le donne pas.
    pub host_name: String,
}

/// The stay's reports, oldest first; none outside a stay.
pub fn load_guest_reports(ctx: &GuestContext) -> Result<Vec<LostFoundReport>> {
    let Some(guest) = ctx.guest.as_ref() else {
        return Ok(Vec::new());
    };
    storage::list_by_stay(guest.session_id)
}

pub fn load_guest_data(ctx: &GuestContext) -> Result<GuestData> {
    let config = ModuleConfig::load(ctx)?;
    Ok(GuestData {
        reports: load_guest_reports(ctx)?,
        window_closed: window_closed(ctx, config.window_days()),
        return_options: config.return_options(),
        shipping_paid_by_guest: config.shipping_paid_by_guest(),
        host_name: ctx
            .host
            .as_ref()
            .map(|host| host.name.trim().to_string())
            .unwrap_or_default(),
    })
}

/// Le délai court depuis le départ. Sans date de départ — ou sans horloge — il ne court pas : mieux
/// vaut un formulaire ouvert qu'une porte fermée sur une date qu'on ne connaît pas.
fn window_closed(ctx: &GuestContext, window_days: u32) -> bool {
    let Some(checkout_at) = ctx.stay.as_ref().and_then(|stay| stay.checkout_at) else {
        return false;
    };
    let Ok(now) = time::now() else {
        return false;
    };
    now > checkout_at + Duration::days(i64::from(window_days))
}
