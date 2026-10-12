//! Bouton du séjour (spec Pré-arrivée §1) : « Renvoyer le formulaire ». Le corps de la modale
//! seulement — titre et Annuler sont au dashboard : le canal (l'e-mail, seul possible), l'aperçu
//! de l'e-mail, puis Renvoyer. Rien à renvoyer quand le voyageur a répondu (§9 cas 1), quand
//! l'arrivée a fermé le formulaire, ou quand il ne lui est pas encore proposé.

use chrono::{DateTime, Utc};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Button, Field, Form, Page, Text};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;
use uuid::Uuid;

use crate::commands::RESEND_FORM;
use crate::config::ModuleConfig;
use crate::show_when::{is_editable_until_checkin, is_form_available};
use crate::storage;

#[portaki_sdk::surface(
    host,
    id = "resend",
    placement = HostPlacement::StayAction,
    label_key = "catalog.host.resend",
    icon = IconName::Mail
)]
pub fn render_host_resend(ctx: HostContext) -> Result<Surface> {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());
    let body: Component = match stay_id {
        None => caption("i18n:host.stay.missingStay"),
        Some(stay_id) => {
            let config = ModuleConfig::load(&ctx)?;
            match blocked(stay_id, &config, checkin(&ctx), time::now()?) {
                Some(reason) => caption(reason),
                None => form(stay_id, &config, &ctx.locale).into(),
            }
        }
    };
    Ok(Surface::new(Page::new().child(body)).with_id(RESEND))
}

/// Pourquoi il n'y a rien à renvoyer, ou `None` quand l'e-mail peut partir.
fn blocked(
    stay_id: Uuid,
    config: &ModuleConfig,
    checkin: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> Option<&'static str> {
    if storage::find_by_stay(stay_id).ok().flatten().is_some() {
        return Some("i18n:host.resend.done");
    }
    if !is_editable_until_checkin(now, checkin) {
        return Some("i18n:host.resend.closed");
    }
    if !is_form_available(config.show_when, now, checkin) {
        return Some("i18n:host.resend.notOpen");
    }
    None
}

fn form(stay_id: Uuid, config: &ModuleConfig, locale: &str) -> Form {
    let mail = crate::email_send::form_available_content(config);
    Form::new()
        .child(
            Field::new()
                .name("channel")
                .label("i18n:host.resend.channel.label")
                .child(Text::new().text("i18n:host.resend.channel.email")),
        )
        .child(
            Field::new()
                .name("preview")
                .label("i18n:host.resend.preview.label")
                .child(
                    Text::new()
                        .text(mail.subject.resolve(locale))
                        .variant(TextVariant::Body),
                )
                .child(
                    Text::new()
                        .text(mail.body.resolve(locale))
                        .variant(TextVariant::Caption),
                ),
        )
        .child(
            Button::new()
                .label("i18n:host.resend.submit")
                .tone(Tone::Primary)
                .action(crate::ids::module_id().command(RESEND_FORM, json!({ "stayId": stay_id }))),
        )
}

fn caption(key: &str) -> Component {
    Text::new().text(key).variant(TextVariant::Caption).into()
}

/// `input.stay.checkIn` : la fenêtre que la plateforme joint à une surface de fiche séjour.
fn checkin(ctx: &HostContext) -> Option<DateTime<Utc>> {
    ctx.input
        .get("stay")?
        .get("checkIn")?
        .as_str()
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|at| at.with_timezone(&Utc))
}
