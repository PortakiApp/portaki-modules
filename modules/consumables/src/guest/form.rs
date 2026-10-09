//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, ChoiceList, Field, FieldHint, Form, Icon, InfoBanner, Stack, Text, TextArea,
};
use portaki_sdk::sdui::surface::Surface;

use super::load::{load_guest_consumables, GuestConsumablesData};
use crate::labels;
use crate::level;

/// Bottom-sheet consumables shortage form (inputs live here — not on the home card).
#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "consumables/form",
    label_key = "nav.consumables"
)]
pub fn render_guest_form(ctx: GuestContext) -> Result<Surface> {
    Ok(match load_guest_consumables(&ctx)? {
        Some(data) => build_form_surface(&data),
        None => super::empty_catalog_card(GUEST_FORM),
    })
}

pub fn build_form_surface(data: &GuestConsumablesData) -> Surface {
    Surface::new(build_form(data)).with_id(GUEST_FORM)
}

/// L'en-tête de la feuille (§5) : une question au voyageur, et ce que l'hôte en fera.
///
/// Le livret le sort du formulaire pour le poser dans la chrome du panneau. Il reste dans
/// l'arbre parce que c'est le module qui l'écrit — la coquille ne connaît que le libellé de
/// navigation, « Consommables », qui n'est pas une question.
fn sheet_header() -> Component {
    Component::Stack(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(12.0)
            .child(
                Icon::new()
                    .name(IconName::Package)
                    .size(24.0)
                    .tone(Tone::Primary),
            )
            .child(
                Stack::new()
                    .gap(4.0)
                    .child(
                        Text::new()
                            .text("i18n:form.head.title")
                            .variant(TextVariant::Title),
                    )
                    .child(
                        Text::new()
                            .text("i18n:form.head.lead")
                            .variant(TextVariant::Caption)
                            .emphasis(Emphasis::Subtle),
                    ),
            ),
    )
}

fn build_form(data: &GuestConsumablesData) -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);
    Form::new()
        .child(sheet_header())
        .child(
            Field::new()
                .name("itemIds")
                .label("i18n:form.item.label")
                .required(true)
                .child(item_choice_list(data)),
        )
        // Le choix multiple n'est pas évident dans une grille de tuiles : un voyageur qui a
        // besoin de café et de sacs poubelle envoyait deux signalements.
        .child(FieldHint::new().text("i18n:form.item.hint"))
        .child(
            Field::new()
                .name("level")
                .label("i18n:form.level.label")
                .required(true)
                .child(level_choice_list()),
        )
        .child(
            Field::new()
                .name("note")
                .label("i18n:form.note.label")
                .child(
                    TextArea::new()
                        .name("note")
                        .placeholder("i18n:form.note.placeholder"),
                ),
        )
        .child(
            Button::new()
                .label("i18n:form.submit")
                .action(submit_action),
        )
        // Le délai que l'hôte a annoncé, et à défaut ce que l'envoi déclenche de toute façon : un
        // hôte qui n'a rien promis ne doit pas laisser le voyageur se demander si quelqu'un l'a lu.
        .child(InfoBanner::new().tone(Tone::Neutral).message(notice(data)))
}

/// « Claire réapprovisionne sous 24 h », ou la phrase générique du module.
fn notice(data: &GuestConsumablesData) -> String {
    let Some(delay) = data.restock_delay.as_deref().map(str::trim) else {
        return "i18n:form.notice".to_string();
    };
    if delay.is_empty() {
        return "i18n:form.notice".to_string();
    }
    match data.host_name.trim() {
        "" => t!("form.notice.delay", delay = delay).unwrap_or_else(|_| "i18n:form.notice".into()),
        host => t!(
            "form.notice.delay.named",
            host = host.to_string(),
            delay = delay
        )
        .unwrap_or_else(|_| "i18n:form.notice".into()),
    }
}

/// Le niveau d'un signalement encore ouvert pour ce produit, s'il y en a un.
///
/// Seuls les signalements pas encore livrés (à traiter ou prévus) comptent : une fois l'hôte passé, le produit redevient un produit
/// comme un autre, et le redemander est légitime.
fn open_report_level(data: &GuestConsumablesData, item_id: uuid::Uuid) -> Option<&'static str> {
    data.reports
        .iter()
        .find(|report| report.item_id == item_id && crate::status::is_pending(&report.status))
        .map(|report| match report.level.as_str() {
            "missing" => "i18n:form.item.reported.missing",
            _ => "i18n:form.item.reported.low",
        })
}

/// Rien n'est présélectionné : en choix multiple, une case déjà cochée part au signalement sans
/// que le voyageur l'ait voulu, et c'est l'hôte qui se déplace pour rien.
fn item_choice_list(data: &GuestConsumablesData) -> ChoiceList {
    let choices: Vec<ChoiceOption> = data
        .items
        .iter()
        .map(|item| {
            let label = labels::pick_label(
                &labels::labels_from_item(item),
                &data.locale,
                &data.property_locale,
            );
            let mut option = ChoiceOption::new(item.id.to_string(), label);
            // Déjà signalé pendant ce séjour : la tuile le dit (§2.5). Sans cela, le voyageur
            // qui rouvre le formulaire ne voit aucune différence, le resignale, et l'hôte se
            // déplace deux fois pour le même paquet de café.
            if let Some(level) = open_report_level(data, item.id) {
                option = option.description(level);
            }
            // L'emoji de l'hôte quand il en a choisi un, le colis du vocabulaire sinon : une
            // grille de huit colis identiques ne se lit pas d'un coup d'œil (§2.5).
            let emoji = item.emoji.trim();
            if emoji.is_empty() {
                option.icon(IconName::Package)
            } else {
                option.emoji(emoji.to_string())
            }
        })
        .collect();

    // Grille et choix multiple (§2.5). Un voyageur qui constate qu'il manque le papier *et* le café
    // le disait en deux envois ; la liste compacte à choix unique l'y obligeait.
    ChoiceList::new()
        .name("itemIds")
        .layout(ChoiceListLayout::Grid)
        .multi(true)
        .choices(choices)
}

fn level_choice_list() -> ChoiceList {
    // Deux choix courts côte à côte : c'est ce que le §2.5 appelle des segments.
    ChoiceList::new()
        .name("level")
        .layout(ChoiceListLayout::Segmented)
        .value(level::DEFAULT)
        .choices(vec![
            // « Il n'y en a plus » porte le ton d'alerte du dessin : les deux segments ne
            // demandent pas la même chose à l'hôte.
            ChoiceOption::new("missing", "i18n:form.level.missing").icon(IconName::DangerTriangle),
            ChoiceOption::new("low", "i18n:form.level.low").icon(IconName::Gauge),
        ])
}
