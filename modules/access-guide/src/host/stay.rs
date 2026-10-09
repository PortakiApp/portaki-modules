//! L'encart du séjour (spec Accès §1) : « Boîte à clés · codes révélés le 23/08/2026 à 16:00 »,
//! ou « Code manquant » quand la méthode en demande un et que l'hôte ne l'a pas saisi. Jamais le
//! code lui-même : l'encart dit quand le voyageur le voit.

use chrono::{DateTime, Utc};
use portaki_sdk::host::i18n::{translate, Vars};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Card, InlineNotice, Page, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, RevealPolicy};
use crate::guest::method_key;
use crate::reveal::{evaluate_reveal, format_available_from};

#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::Key
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::read(&ctx)?;
    let mut card = Card::new()
        .icon(IconName::Key)
        .title("i18n:host.stay.title");
    if config.entry_code_missing() {
        card = card.child(
            InlineNotice::new()
                .message("i18n:host.stay.missing")
                .tone(Tone::Warning),
        );
    } else {
        let method = method_key(config.primary_method);
        let method = translate(method, &Vars::new()).unwrap_or_else(|_| method.to_string());
        let line = match (config.has_any_code(), window(&ctx)) {
            (false, _) => method,
            (true, Some((checkin, checkout))) => format!(
                "{method} · {}",
                reveal_line(
                    &config,
                    checkin,
                    checkout,
                    time::now()?,
                    &ctx.property.timezone
                )
            ),
            // Pas de fenêtre : la méthode seule plutôt qu'une date inventée.
            (true, None) => method,
        };
        card = card.child(Text::new().text(line).variant(TextVariant::Body));
    }
    Ok(Surface::new(Page::new().child(card)).with_id(STAY))
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

/// « codes révélés le 23/08/2026 à 16:00 », « codes masqués jusqu'au … », ou la règle quand il n'y
/// a pas de date : dès la réservation, séjour terminé.
pub(crate) fn reveal_line(
    config: &ModuleConfig,
    checkin: DateTime<Utc>,
    checkout: DateTime<Utc>,
    now: DateTime<Utc>,
    timezone: &str,
) -> String {
    if config.reveal_policy == RevealPolicy::Always {
        return text("host.stay.always", &[]);
    }
    let decision = evaluate_reveal(
        config.reveal_policy,
        now,
        Some(checkin),
        Some(checkout),
        timezone,
    );
    if decision.ended {
        return text("host.stay.ended", &[]);
    }
    let Some(at) = decision.available_from else {
        return text("host.stay.always", &[]);
    };
    let when = format_available_from(at, timezone);
    let key = if decision.revealed {
        "host.stay.revealed"
    } else {
        "host.stay.hidden"
    };
    text(key, &[("when", &when)])
}

fn text(key: &str, vars: &[(&str, &str)]) -> String {
    let mut bag = Vars::new();
    for (name, value) in vars {
        bag.set(*name, *value);
    }
    translate(key, &bag).unwrap_or_else(|_| key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn the_line_follows_the_reveal_and_the_departure() {
        let config = ModuleConfig::default(); // la veille à 16 h
        let (checkin, checkout) = (at("2026-08-24T14:00:00Z"), at("2026-08-29T08:00:00Z"));
        let line = |now| reveal_line(&config, checkin, checkout, at(now), "Europe/Paris");
        // Hors du Wasm, `translate` rend la clé : c'est la clé choisie qu'on vérifie.
        assert_eq!(line("2026-08-20T10:00:00Z"), "host.stay.hidden");
        assert_eq!(line("2026-08-23T15:00:00Z"), "host.stay.revealed");
        assert_eq!(line("2026-08-30T10:00:00Z"), "host.stay.ended");
        let always = ModuleConfig {
            reveal_policy: RevealPolicy::Always,
            ..ModuleConfig::default()
        };
        assert_eq!(
            reveal_line(
                &always,
                checkin,
                checkout,
                at("2026-08-20T10:00:00Z"),
                "UTC"
            ),
            "host.stay.always"
        );
    }
}
