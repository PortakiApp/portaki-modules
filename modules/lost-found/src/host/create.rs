//! Stay declare-found form — mirrors design `foundObjectModal` / `sheetFound`.
//!
//! Hosted in a stay-action modal: chrome (title / icon / guest context / Annuler)
//! is owned by the dashboard shell; this surface is the form body only.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Field, FieldHint, Form, Page, RichTextEditor, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;
use uuid::Uuid;

use crate::commands::SubmitFoundArgs;

/// Builds the declare-found form body (no page chrome — stay-action modal owns it).
///
/// - `input.stayId` — required target stay.
/// - Status is never shown; [`submit_found`](crate::submit_found) always uses
///   `to_collect`.
/// - Description is TipTap [`RichTextEditor`] (same host primitive as main note).
pub(crate) fn build_create_found_form(ctx: &HostContext) -> Component {
    let stay_id = ctx
        .input_str("stayId")
        .and_then(|raw| Uuid::parse_str(raw).ok());

    let Some(stay_id) = stay_id else {
        return Component::Stack(Stack::new().gap(12.0).children(vec![Text::new()
            .text("i18n:host.stay.missingStay")
            .variant(TextVariant::Caption)
            .into()]));
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

    Form::new()
        .child(
            Field::new()
                .name("description")
                .label("i18n:host.create.description.label")
                .required(true)
                .child(RichTextEditor::new().name("description").value("")),
        )
        .child(FieldHint::new().text("i18n:host.create.hint"))
        .child(
            Button::new()
                .label("i18n:host.create.submit")
                .tone(Tone::Primary)
                .action(submit_action),
        )
        .into()
}

/// Stay-action modal body — declare a found item for one stay.
#[portaki_sdk::surface(host, id = "create")]
pub fn render_host_create(ctx: HostContext) -> Surface {
    Surface::new(Page::new().child(build_create_found_form(&ctx))).with_id(crate::ids::HOST_CREATE)
}
