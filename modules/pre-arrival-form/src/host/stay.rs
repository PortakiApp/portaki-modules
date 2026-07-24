//! Stay-scoped host surface — design stay detail « Formulaire de pré-arrivée ».
//!
//! Layout (Dashboard.dc.html): title + status pill in the card header, then either
//! pending copy or detail rows for enabled / answered questions.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{Card, ListItem, Page, Pill, Stack, Text};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::config::load_config;
use crate::entities::PreArrivalResponse;
use crate::storage;

/// Stay detail embed — read-only pre-arrival responses for `input.stayId`.
#[portaki_sdk::surface(host, id = "stay")]
pub fn render_host_stay(ctx: HostContext) -> Surface {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());

    let body = match stay_id {
        None => missing_stay_card(),
        Some(stay_id) => match storage::find_by_stay(stay_id).ok().flatten() {
            Some(row) => completed_card(&row),
            None => pending_card(),
        },
    };

    Surface::new(Page::new().child(body)).with_id(crate::ids::HOST_STAY)
}

fn missing_stay_card() -> Component {
    Component::Card(
        Card::new()
            .icon("clipboard")
            .title("i18n:surface.host.stay.title")
            .child(
                Text::new()
                    .text("i18n:host.stay.missingStay")
                    .variant(TextVariant::Caption),
            ),
    )
}

fn pending_card() -> Component {
    let status = Pill::new()
        .label("i18n:host.stay.status.pending")
        .tone(Tone::Neutral);

    Component::Card(
        Card::new()
            .icon("clipboard")
            .title("i18n:surface.host.stay.title")
            .child(status)
            .child(
                Text::new()
                    .text("i18n:host.stay.pending")
                    .variant(TextVariant::Caption),
            ),
    )
}

fn completed_card(row: &PreArrivalResponse) -> Component {
    let status = Pill::new()
        .label("i18n:host.stay.status.done")
        .tone(Tone::Success);
    let questions = load_config().unwrap_or_default().questions;

    let mut rows: Vec<Component> = Vec::new();

    if questions.ask_arrival_time {
        rows.push(detail_row(
            "clock-circle",
            "i18n:host.stay.arrival.label",
            display_or_dash(row.arrival_time.as_deref()),
            None,
        ));
    }
    if questions.ask_occasion {
        rows.push(detail_row(
            "star",
            "i18n:host.stay.occasion.label",
            display_or_dash(row.occasion.as_deref()),
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
            .icon("clipboard")
            .title("i18n:surface.host.stay.title")
            .child(status)
            .child(Stack::new().gap(0.0).children(rows)),
    )
}

fn detail_row(leading: &str, label_i18n: &str, value: String, tone: Option<Tone>) -> Component {
    let mut item = ListItem::new()
        .title(label_i18n)
        .subtitle(value)
        .leading(leading)
        .chevron(false);
    if let Some(tone) = tone {
        item = item.tone(tone);
    }
    Component::ListItem(item)
}

fn display_or_dash(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("—")
        .to_string()
}
