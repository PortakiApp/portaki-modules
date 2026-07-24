//! Stay declare-found form — mirrors design `foundObjectModal` / `sheetFound`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Field, FieldHint, Form, HeaderTitle, Stack, Text, TextArea,
};
use uuid::Uuid;

use crate::commands::SubmitFoundArgs;

/// Builds the declare-found form (stay modal layout).
///
/// - `input.stayId` — required target stay.
/// - `input.guestName` / `input.stayDates` — header context (optional).
/// - Status is never shown; [`submit_found`](crate::submit_found) always uses
///   `to_collect`.
pub(crate) fn build_create_found_form(ctx: &HostContext) -> Component {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());

    let guest_name = ctx.input_str("guestName").unwrap_or("");
    let stay_dates = ctx.input_str("stayDates").unwrap_or("");
    let header_sub = match (guest_name.is_empty(), stay_dates.is_empty()) {
        (true, true) => None,
        (false, true) => Some(guest_name.to_string()),
        (true, false) => Some(stay_dates.to_string()),
        (false, false) => Some(format!("{guest_name} · {stay_dates}")),
    };

    let mut header = HeaderTitle::new().title("i18n:host.create.title");
    if let Some(sub) = header_sub {
        header = header.subtitle(sub);
    }

    let mut children: Vec<Component> = vec![header.into()];

    let Some(stay_id) = stay_id else {
        children.push(
            Text::new()
                .text("i18n:host.stay.missingStay")
                .variant(TextVariant::Caption)
                .into(),
        );
        return Component::Stack(Stack::new().gap(12.0).children(children));
    };

    let submit_action = crate::ids::module_id().command(
        crate::ids::SUBMIT_FOUND,
        SubmitFoundArgs {
            stay_ids: vec![stay_id],
            stay_id: Some(stay_id),
            description: String::new(),
            status: None,
        },
    );

    children.push(
        Form::new()
            .child(
                Field::new()
                    .name("description")
                    .label("i18n:host.create.description.label")
                    .required(true)
                    .child(
                        TextArea::new()
                            .name("description")
                            .placeholder("i18n:host.create.description.placeholder"),
                    ),
            )
            .child(FieldHint::new().text("i18n:host.create.hint"))
            .child(
                Button::new()
                    .label("i18n:host.create.submit")
                    .tone(Tone::Primary)
                    .action(submit_action),
            )
            .into(),
    );

    Component::Stack(Stack::new().gap(12.0).children(children))
}
