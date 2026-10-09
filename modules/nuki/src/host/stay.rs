//! Stay detail encart (spec Nuki §1) : « Accès Nuki actif du 24/08 16:00 au 29/08 10:00 » ;
//! « Code 482913 ». Le code est lu sur la serrure, jamais créé ici : il naît au premier affichage
//! par le voyageur.

use portaki_sdk::host::time::PropertyTz;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use crate::stay_code;

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::Lock
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let lines: Vec<Component> = lines(&ctx, &config)?
        .into_iter()
        .map(|line| Text::new().text(line).variant(TextVariant::Body).into())
        .collect();
    let card = Card::new()
        .icon(IconName::Lock)
        .title("i18n:host.stay.title")
        .children(lines);
    Ok(Surface::new(Page::new().child(card)).with_id(STAY))
}

fn lines(ctx: &HostContext, config: &ModuleConfig) -> Result<Vec<String>> {
    if !stay_code::wanted(ctx, config) {
        return Ok(vec!["i18n:host.stay.shared".into()]);
    }
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let (Some(stay_id), Some((checkin, checkout))) = (stay_id, window(ctx)) else {
        return Ok(vec!["i18n:host.stay.unknown".into()]);
    };
    let tz = PropertyTz::parse(&ctx.property.timezone);
    let local = |at: DateTime<Utc>| match tz {
        Some(tz) => tz.to_local(at).format("%d/%m %H:%M").to_string(),
        None => at.format("%d/%m %H:%M").to_string(),
    };
    let window_line = t!(
        "host.stay.window",
        from = &local(checkin),
        until = &local(checkout)
    )?;
    let code_line = match stay_code::find(config.smartlock_id_trimmed(), &stay_id) {
        Ok(Some(code)) => t!("host.stay.code", code = &code)?,
        Ok(None) => "i18n:host.stay.pending".into(),
        Err(_) => "i18n:host.stay.offline".into(),
    };
    Ok(vec![window_line, code_line])
}

/// `input.stay` : la fenêtre que la plateforme joint à une surface de fiche séjour.
fn window(ctx: &HostContext) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let stay = ctx.input.get("stay")?;
    let parse = |key: &str| {
        stay.get(key)
            .and_then(|v| v.as_str())
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
            .map(|at| at.with_timezone(&Utc))
    };
    Some((parse("checkIn")?, parse("checkOut")?))
}
