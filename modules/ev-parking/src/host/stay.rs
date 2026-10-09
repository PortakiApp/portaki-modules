//! Stay detail encart (spec Parking VE §1) : « Place 8 · borne Type 2 7 kW ». Jamais un code —
//! l'hôte les connaît ; l'encart rappelle seulement la place et la borne.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::Zap
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let spot = config.spot_label.host_value(&ctx);
    let text = if spot.trim().is_empty() {
        "i18n:host.stay.noSpot".to_string()
    } else {
        let key = format!("host.chargerType.label.{}", config.charger_type());
        let charger = t!(&key).unwrap_or_default();
        let charger = t!("host.stay.charger", charger = &charger).unwrap_or_default();
        line(spot.trim(), &charger, config.power())
    };
    let card = Card::new()
        .icon(IconName::Zap)
        .title("i18n:host.stay.title")
        .child(Text::new().text(text).variant(TextVariant::Body));
    Ok(Surface::new(Page::new().child(card)).with_id(STAY))
}

/// La place, puis la borne et sa puissance quand elle est renseignée (même écriture que le livret).
fn line(spot: &str, charger: &str, power: Option<f64>) -> String {
    match power {
        Some(kw) => format!("{spot} · {charger} {kw} kW"),
        None => format!("{spot} · {charger}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spot_then_the_charger_and_its_power() {
        assert_eq!(
            line("Place 8", "borne Type 2", Some(7.0)),
            "Place 8 · borne Type 2 7 kW"
        );
        assert_eq!(
            line("Place 8", "borne Type 2", Some(7.4)),
            "Place 8 · borne Type 2 7.4 kW"
        );
        assert_eq!(line("Place 8", "borne CCS", None), "Place 8 · borne CCS");
    }
}
