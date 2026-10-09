//! Stay-scoped host surface — design stay detail « Formulaire de pré-arrivée ».
//!
//! Layout (Dashboard.dc.html): title + status pill in the card header, then either
//! pending copy or detail rows for enabled / answered questions.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Leading;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Card, ListItem, Page, Pill, Stack, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::config::ModuleConfig;
use crate::entities::PreArrivalResponse;
use crate::storage;

/// Stay detail embed — read-only pre-arrival responses for `input.stayId`.
#[portaki_sdk::surface(
    host,
    id = "stay",
    placement = HostPlacement::StayDetail,
    label_key = "catalog.host.stay",
    icon = IconName::Clipboard
)]
pub fn render_host_stay(ctx: HostContext) -> Result<Surface> {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());

    let body = match stay_id {
        None => missing_stay_card(),
        Some(stay_id) => match storage::find_by_stay(stay_id).ok().flatten() {
            Some(row) => completed_card(&row, &ModuleConfig::load(&ctx)?),
            None => pending_card(ModuleConfig::load(&ctx)?.reminder),
        },
    };

    Ok(Surface::new(Page::new().child(body)).with_id(STAY))
}

fn missing_stay_card() -> Component {
    Component::Card(
        Card::new()
            .icon(IconName::Clipboard)
            .title("i18n:surface.host.stay.title")
            .child(
                Text::new()
                    .text("i18n:host.stay.missingStay")
                    .variant(TextVariant::Caption),
            ),
    )
}

fn pending_card(reminder: bool) -> Component {
    let status = Pill::new()
        .label("i18n:host.stay.status.pending")
        .tone(Tone::Neutral);

    Component::Card(
        Card::new()
            .icon(IconName::Clipboard)
            .title("i18n:surface.host.stay.title")
            .child(status)
            .child(
                Text::new()
                    .text(if reminder {
                        "i18n:host.stay.pending.reminder"
                    } else {
                        "i18n:host.stay.pending"
                    })
                    .variant(TextVariant::Caption),
            ),
    )
}

fn completed_card(row: &PreArrivalResponse, questions: &ModuleConfig) -> Component {
    let status = Pill::new()
        .label("i18n:host.stay.status.done")
        .tone(Tone::Success);

    let mut rows: Vec<Component> = Vec::new();

    if questions.ask_arrival_time {
        rows.push(detail_row(
            "clock-circle",
            "i18n:host.stay.arrival.label",
            // « dès 17 h », pas « 17:00 ». Le voyageur choisit un créneau et le formulaire en
            // envoie le début : lu comme une heure exacte, l'hôte attend à 17 h pile quelqu'un
            // qui a annoncé 17–19 h.
            match row.arrival_time.as_deref() {
                Some(time) => crate::slots::floor_label(time),
                None => display_or_dash(None),
            },
            None,
        ));
    }
    if questions.ask_occasion {
        rows.push(detail_row(
            "star",
            "i18n:host.stay.occasion.label",
            // La valeur stockée est une clé de liste depuis que l'occasion se choisit : l'hôte
            // doit lire « Anniversaire », pas `birthday`. Une réponse d'avant la liste est du
            // texte libre et s'affiche telle quelle.
            from_choice(row.occasion.as_deref(), &crate::OCCASIONS, "form.occasion"),
            None,
        ));
    }
    if questions.ask_transport {
        rows.push(detail_row(
            "car",
            "i18n:host.stay.transport.label",
            from_choice(
                row.transport.as_deref(),
                &crate::TRANSPORTS,
                "form.transport",
            ),
            None,
        ));
    }
    if questions.ask_allergies {
        let allergies_raw = row
            .allergies
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        let allergies = allergies_raw
            .map(|value| value.to_string())
            .unwrap_or_else(|| "i18n:host.stay.allergies.none".to_string());
        let allergies_tone = allergies_raw.map(|_| Tone::Warning);
        rows.push(detail_row(
            "danger-triangle",
            "i18n:host.stay.allergies.label",
            allergies,
            allergies_tone,
        ));
    }
    if questions.ask_guest_count {
        rows.push(detail_row(
            "users",
            "i18n:host.stay.guestCount.label",
            display_or_dash(row.guest_count.as_deref()),
            None,
        ));
    }
    if questions.ask_special_needs {
        rows.push(detail_row(
            "home",
            "i18n:host.stay.specialNeeds.label",
            display_or_dash(row.special_needs.as_deref()),
            None,
        ));
    }
    if questions.ask_id_document {
        rows.push(detail_row(
            "clipboard",
            "i18n:host.stay.idDocument.label",
            display_or_dash(row.id_document.as_deref()),
            None,
        ));
    }

    if let Some(message) = row
        .guest_message
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        rows.push(detail_row(
            "message",
            "i18n:host.stay.message.label",
            message.to_string(),
            None,
        ));
    }

    Component::Card(
        Card::new()
            .icon(IconName::Clipboard)
            .title("i18n:surface.host.stay.title")
            .child(status)
            .child(Stack::new().gap(0.0).children(rows)),
    )
}

fn detail_row(leading: &str, label_i18n: &str, value: String, tone: Option<Tone>) -> Component {
    let mut item = ListItem::new()
        .title(label_i18n)
        .subtitle(value)
        .leading(Leading::Icon(leading.into()))
        .chevron(false);
    if let Some(tone) = tone {
        item = item.tone(tone);
    }
    Component::ListItem(item)
}

/// Une valeur de liste, rendue dans la langue de l'hôte.
///
/// Hors liste — une réponse écrite avant que la question devienne un choix —, la valeur s'affiche
/// telle quelle : c'est ce que le voyageur avait écrit, et le perdre serait pire que l'afficher.
fn from_choice(value: Option<&str>, choices: &[&str], prefix: &str) -> String {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return "—".to_string();
    };
    if !choices.contains(&value) {
        return value.to_string();
    }
    t!(&format!("{prefix}.{value}")).unwrap_or_else(|_| value.to_string())
}

fn display_or_dash(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("—")
        .to_string()
}
