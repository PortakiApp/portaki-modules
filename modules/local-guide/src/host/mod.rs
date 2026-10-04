//! Host dashboard surface — design `guide-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Field, FieldHint, Form, ImageUpload, InfoBanner, Page, Select, Stack,
    StepList, Text, TextArea, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::affiliate::{looks_like_url, normalize_curated_url, CuratedUrlError, MAX_CURATED_LINKS};
use crate::config::{ActivityRow, ModuleConfig, SpotRow, TIQETS_RADIUS_CHOICES_KM};
use crate::tiqets::TiqetsStatus;
use crate::viator::ViatorStatus;

/// Combien d'adresses le formulaire accepte.
///
/// Une capacité, pas un nombre de lignes dessinées : six emplacements figés gelaient la liste à
/// six — l'hôte ne pouvait pas en saisir une septième parce que le formulaire ne la dessinait
/// jamais, et voyait quatre cartes vides quand il en avait saisi deux.
pub const MAX_SPOTS: usize = 12;

/// Créneaux d'activités affichés : les lignes stockées, plus un libre.
///
/// Deux au minimum pour que la section ait l'air d'une liste, [`MAX_CURATED_LINKS`] au
/// maximum — la même borne que celle appliquée au rendu voyageur, pour que le formulaire
/// ne propose jamais un créneau qui ne s'afficherait pas. Une ligne stockée au-delà reste
/// affichée : le formulaire ne la retire pas en silence.
fn activity_slots(stored: usize) -> usize {
    (stored + 1).clamp(2, MAX_CURATED_LINKS).max(stored)
}

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::GuideEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::MapPin
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let disclaimer = config.disclaimer.host_value(&ctx);
    let activities = config.activities();
    let tiqets = config.tiqets();
    let viator = config.viator();

    let activities_enabled = ctx.input_bool("activities_enabled", activities.enabled);
    let activities_destination = ctx
        .input_str("activities_destination")
        .map(str::to_string)
        .unwrap_or_else(|| activities.destination.clone());
    let activities_intro = ctx
        .input_str("activities_intro")
        .map(str::to_string)
        .unwrap_or_else(|| activities.intro.host_value(&ctx).to_string());

    let tiqets_enabled = ctx.input_bool("tiqets_enabled", tiqets.enabled);
    let tiqets_radius = ctx
        .input_str("tiqets_radius_km")
        .map(str::to_string)
        .unwrap_or_else(|| tiqets.normalized_radius_km().to_string());
    let tiqets_min_rating = ctx
        .input_str("tiqets_min_rating")
        .map(str::to_string)
        .unwrap_or_else(|| tiqets.min_rating.to_string());
    let tiqets_status = crate::tiqets::status(
        &ctx,
        &crate::config::TiqetsConfig {
            enabled: tiqets_enabled,
            ..tiqets
        },
    );

    let viator_enabled = ctx.input_bool("viator_enabled", viator.enabled);
    let viator_min_rating = ctx
        .input_str("viator_min_rating")
        .map(str::to_string)
        .unwrap_or_else(|| viator.min_rating.to_string());
    let viator_status = crate::viator::status(
        &ctx,
        &crate::config::ViatorConfig {
            enabled: viator_enabled,
            ..viator
        },
    );

    let mut cards: Vec<Component> = vec![spots_card(&config, &ctx)];
    cards.push(activities_card(
        &ctx,
        activities_enabled,
        &activities_destination,
        &activities_intro,
        &config.activities,
    ));
    cards.push(tiqets_card(
        tiqets_enabled,
        &tiqets_radius,
        &tiqets_min_rating,
        tiqets_status,
    ));
    cards.push(viator_card(
        viator_enabled,
        &viator_min_rating,
        viator_status,
    ));
    cards.push(
        Card::new()
            .title("i18n:host.section.disclaimer")
            .icon(IconName::InfoCircle)
            .children(vec![Field::new()
                .name("disclaimer")
                .label("i18n:host.disclaimer.label")
                .child(
                    TextArea::new()
                        .name("disclaimer")
                        .value(disclaimer)
                        .placeholder("i18n:host.disclaimer.placeholder"),
                )
                .into()])
            .into(),
    );
    // Pas de bouton Enregistrer : le tableau de bord enregistre le formulaire à la saisie.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]))),
    )
    .with_id(MAIN))
}

/// Carte « Activités & billets ».
///
/// Tous les champs sont facultatifs : sans rien, la section tourne déjà sur la ville de
/// l'adresse du logement.
fn activities_card(
    ctx: &HostContext,
    enabled: bool,
    destination: &str,
    intro: &str,
    links: &[ActivityRow],
) -> Component {
    let slots = activity_slots(links.len());
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("activities_enabled")
        .label("i18n:host.activities.enabled")
        .icon(IconName::Ticket)
        .checked(enabled)
        .into()];

    let mut rows: Vec<Component> = Vec::new();
    let mut has_rejected_url = false;
    for index in 0..slots {
        let stored = links.get(index);
        let url = ctx
            .input_str(&format!("activities.{index}.url"))
            .map(str::to_string)
            .unwrap_or_else(|| stored.map(|row| row.url.clone()).unwrap_or_default());
        let label = ctx
            .input_str(&format!("activities.{index}.label"))
            .map(str::to_string)
            .unwrap_or_else(|| {
                stored
                    .map(|row| row.label.host_value(ctx).to_string())
                    .unwrap_or_default()
            });

        if matches!(
            normalize_curated_url(&url),
            Err(CuratedUrlError::NotGetYourGuide)
        ) {
            has_rejected_url = true;
        }

        // A filled row sends its id, so a save merges into it (and keeps its label's other
        // languages). A blank slot has nothing to keep — and an id would make it count as filled.
        rows.extend(
            stored
                .filter(|row| !row.is_blank())
                .map(|row| sdui::row_id("activities", index, Some(&row.id))),
        );
        rows.push(
            Field::new()
                .name(format!("activities.{index}.url"))
                .label("i18n:host.activities.link.url")
                .child(
                    TextInput::new()
                        .name(format!("activities.{index}.url"))
                        .value(url)
                        .placeholder("i18n:host.activities.link.url.placeholder"),
                )
                .into(),
        );
        rows.push(
            Field::new()
                .name(format!("activities.{index}.label"))
                .label("i18n:host.activities.link.label")
                .child(
                    TextInput::new()
                        .name(format!("activities.{index}.label"))
                        .value(label)
                        .placeholder("i18n:host.activities.link.label.placeholder"),
                )
                .into(),
        );
    }

    // La destination accepte une URL : quand c'en est une, elle est relevée ici comme les
    // liens de la liste, avec le même message.
    if looks_like_url(destination)
        && matches!(
            normalize_curated_url(destination),
            Err(CuratedUrlError::NotGetYourGuide)
        )
    {
        has_rejected_url = true;
    }

    // Même formulation que le refus de la commande, mais lisible avant d'avoir cliqué
    // sur Enregistrer.
    if has_rejected_url {
        children.push(
            InfoBanner::new()
                .tone(Tone::Warning)
                .message("i18n:host.activities.error.badUrl")
                .into(),
        );
    }

    children.push(
        Field::new()
            .name("activities_destination")
            .label("i18n:host.activities.destination")
            .child(
                TextInput::new()
                    .name("activities_destination")
                    .value(destination.to_string())
                    .placeholder("i18n:host.activities.destination.placeholder"),
            )
            .into(),
    );
    children.push(
        Field::new()
            .name("activities_intro")
            .label("i18n:host.activities.intro")
            .child(
                TextArea::new()
                    .name("activities_intro")
                    .value(intro.to_string())
                    .placeholder("i18n:host.activities.intro.placeholder"),
            )
            .into(),
    );
    children.extend(rows);
    children.push(
        Text::new()
            .text("i18n:host.activities.help")
            .variant(TextVariant::Caption)
            .into(),
    );

    Card::new()
        .title("i18n:host.section.activities")
        .subtitle("i18n:host.section.activities.help")
        .icon(IconName::Ticket)
        .children(children)
        .into()
}

/// Carte « Billets & activités (Tiqets) ».
///
/// L'état dit à l'hôte pourquoi rien ne s'afficherait : section éteinte, pas de clé (ni
/// incluse à l'offre, ni la sienne), ou logement sans position sur la carte.
fn tiqets_card(enabled: bool, radius: &str, min_rating: &str, status: TiqetsStatus) -> Component {
    let status_key = match status {
        TiqetsStatus::Off => "i18n:host.tiqets.status.off",
        TiqetsStatus::MissingKey => "i18n:host.tiqets.status.missingKey",
        TiqetsStatus::MissingCoordinates => "i18n:host.tiqets.status.missingCoordinates",
        TiqetsStatus::Ready => "i18n:host.tiqets.status.ready",
    };
    let radius_options = TIQETS_RADIUS_CHOICES_KM
        .iter()
        .map(|km| ChoiceOption::new(km.to_string(), format!("i18n:host.tiqets.radius.{km}")))
        .collect();

    let mut children: Vec<Component> = vec![
        ToggleRow::new()
            .name("tiqets_enabled")
            .label("i18n:host.tiqets.enabled")
            .icon(IconName::Ticket)
            .checked(enabled)
            .into(),
        Field::new()
            .name("tiqets_radius_km")
            .label("i18n:host.tiqets.radius")
            .child(
                Select::new()
                    .name("tiqets_radius_km")
                    .options(radius_options)
                    .value(radius.to_string()),
            )
            .into(),
        Field::new()
            .name("tiqets_min_rating")
            .label("i18n:host.tiqets.minRating")
            .child(
                Select::new()
                    .name("tiqets_min_rating")
                    .options(vec![
                        ChoiceOption::new("0", "i18n:host.tiqets.minRating.0"),
                        ChoiceOption::new("3", "i18n:host.tiqets.minRating.3"),
                        ChoiceOption::new("4", "i18n:host.tiqets.minRating.4"),
                    ])
                    .value(min_rating.to_string()),
            )
            .into(),
    ];
    if matches!(
        status,
        TiqetsStatus::MissingKey | TiqetsStatus::MissingCoordinates
    ) {
        children.push(
            InfoBanner::new()
                .tone(Tone::Warning)
                .message(status_key)
                .into(),
        );
    } else {
        children.push(
            Text::new()
                .text(status_key)
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    children.push(
        Text::new()
            .text("i18n:host.tiqets.help")
            .variant(TextVariant::Caption)
            .into(),
    );

    Card::new()
        .title("i18n:host.section.tiqets")
        .subtitle("i18n:host.section.tiqets.help")
        .icon(IconName::Ticket)
        .children(children)
        .into()
}

/// Carte « Activités (Viator) ».
///
/// L'état dit à l'hôte pourquoi rien ne s'afficherait : section éteinte, clé de Portaki
/// indisponible (l'hôte ne peut pas en poser une, la licence Viator l'interdit), ou adresse
/// sans ville lisible.
fn viator_card(enabled: bool, min_rating: &str, status: ViatorStatus) -> Component {
    let status_key = match status {
        ViatorStatus::Off => "i18n:host.viator.status.off",
        ViatorStatus::MissingKey => "i18n:host.viator.status.missingKey",
        ViatorStatus::MissingCity => "i18n:host.viator.status.missingCity",
        ViatorStatus::Ready => "i18n:host.viator.status.ready",
    };

    let mut children: Vec<Component> = vec![
        ToggleRow::new()
            .name("viator_enabled")
            .label("i18n:host.viator.enabled")
            .icon(IconName::Ticket)
            .checked(enabled)
            .into(),
        Field::new()
            .name("viator_min_rating")
            .label("i18n:host.viator.minRating")
            .child(
                Select::new()
                    .name("viator_min_rating")
                    .options(vec![
                        ChoiceOption::new("0", "i18n:host.viator.minRating.0"),
                        ChoiceOption::new("3", "i18n:host.viator.minRating.3"),
                        ChoiceOption::new("4", "i18n:host.viator.minRating.4"),
                    ])
                    .value(min_rating.to_string()),
            )
            .into(),
    ];
    if matches!(status, ViatorStatus::MissingKey | ViatorStatus::MissingCity) {
        children.push(
            InfoBanner::new()
                .tone(Tone::Warning)
                .message(status_key)
                .into(),
        );
    } else {
        children.push(
            Text::new()
                .text(status_key)
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    children.push(
        Text::new()
            .text("i18n:host.viator.help")
            .variant(TextVariant::Caption)
            .into(),
    );

    Card::new()
        .title("i18n:host.section.viator")
        .subtitle("i18n:host.section.viator.help")
        .icon(IconName::Ticket)
        .children(children)
        .into()
}

/// Les adresses de l'hôte, en lignes dynamiques bornées.
fn spots_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows_count = draft_rows(ctx, config.spots.len());
    let rows: Vec<Component> = (0..rows_count)
        .map(|index| spot_row(index, config.spots.get(index), ctx))
        .collect();

    Card::new()
        .title("i18n:host.spots.title")
        .subtitle("i18n:host.spots.subtitle")
        .icon(IconName::MapPin)
        .child(
            StepList::new()
                .addLabel("i18n:host.spots.add")
                .removeLabel("i18n:host.spots.remove")
                .emptyTitle("i18n:host.spots.emptyTitle")
                .emptyDescription("i18n:host.spots.emptyDescription")
                .itemKeyPrefix("spots")
                .addAction(emit_input(RowCount {
                    spots_count: (rows_count + 1).min(MAX_SPOTS),
                }))
                .children(rows),
        )
        .into()
}

/// Combien de lignes dessiner : ce que « Ajouter » a demandé, sinon ce qui est stocké, borné.
fn draft_rows(ctx: &HostContext, stored: usize) -> usize {
    match ctx.input_u64("spots_count") {
        Some(asked) => (asked as usize).clamp(1, MAX_SPOTS),
        None => stored.clamp(1, MAX_SPOTS),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    spots_count: usize,
}

fn emit_input(payload: impl serde::Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

/// Une ligne de texte d'une adresse — le motif se répète six fois, autant le nommer une fois.
fn text_field(index: usize, name: &str, label: &str, value: &str) -> Component {
    let field = format!("spots.{index}.{name}");
    Field::new()
        .name(field.clone())
        .label(label)
        .child(TextInput::new().name(field).value(value))
        .into()
}

fn spot_row(index: usize, spot: Option<&SpotRow>, ctx: &HostContext) -> Component {
    let title = spot.map(|s| s.title.host_value(ctx)).unwrap_or_default();
    let category = spot.and_then(|s| s.category.as_deref()).unwrap_or("");
    let distance = spot.and_then(|s| s.distance.as_deref()).unwrap_or("");
    let tag = spot.and_then(|s| s.tag.as_deref()).unwrap_or("");
    let detail = spot.map(|s| s.detail.host_value(ctx)).unwrap_or_default();
    let address = spot.and_then(|s| s.address.as_deref()).unwrap_or("");
    let perk = spot.map(|s| s.perk.host_value(ctx)).unwrap_or_default();
    let note = spot
        .and_then(|s| s.note.as_ref())
        .map(|note| note.host_value(ctx))
        .unwrap_or_default();
    let price = spot.and_then(|s| s.price.as_deref()).unwrap_or("");
    let hours = spot.and_then(|s| s.hours.as_deref()).unwrap_or("");
    let opening = spot.and_then(|s| s.opening.as_deref()).unwrap_or("");
    let parking = spot.and_then(|s| s.parking.as_deref()).unwrap_or("");
    let phone = spot.and_then(|s| s.phone.as_deref()).unwrap_or("");
    let url = spot.and_then(|s| s.url.as_deref()).unwrap_or("");
    let photo = spot.map(|s| s.photo.clone()).unwrap_or_default();
    // Sans position, le sélecteur ne reçoit ni latitude ni longitude : il part vide.
    let mut picker = AddressMapPicker::new()
        .addressName(format!("spots.{index}.address"))
        .latName(format!("spots.{index}.lat"))
        .lngName(format!("spots.{index}.lng"))
        .address(address)
        .label("i18n:host.spot.address")
        .hint("i18n:host.spot.address.hint");
    if let Some((lat, lng)) = spot.and_then(SpotRow::coords) {
        picker = picker.lat(lat).lng(lng);
    }

    // A filled row sends its id, so a save merges into it (and keeps its url, its note, its
    // other languages). A blank slot has nothing to keep — and an id would make it count as filled.
    let id = spot
        .filter(|s| !s.is_blank())
        .map(|s| sdui::row_id("spots", index, Some(&s.id)));

    let fields: Vec<Component> = vec![
        Field::new()
            .name(format!("spots.{index}.title"))
            .label("i18n:host.spot.name")
            .child(
                TextInput::new()
                    .name(format!("spots.{index}.title"))
                    .value(title),
            )
            .into(),
        Field::new()
            .name(format!("spots.{index}.category"))
            .label("i18n:host.spot.category")
            .child(
                TextInput::new()
                    .name(format!("spots.{index}.category"))
                    .value(category),
            )
            .into(),
        Field::new()
            .name(format!("spots.{index}.distance"))
            .label("i18n:host.spot.distance")
            .child(
                TextInput::new()
                    .name(format!("spots.{index}.distance"))
                    .value(distance),
            )
            .into(),
        Field::new()
            .name(format!("spots.{index}.tag"))
            .label("i18n:host.spot.tag")
            .child(
                TextInput::new()
                    .name(format!("spots.{index}.tag"))
                    .value(tag),
            )
            .into(),
        Field::new()
            .name(format!("spots.{index}.detail"))
            .label("i18n:host.spot.description")
            .child(
                TextArea::new()
                    .name(format!("spots.{index}.detail"))
                    .value(detail),
            )
            .into(),
        // L'avantage en entier, à côté du badge : « −10 % » tient sur une tuile, « 10 % sur le
        // menu du soir, sur présentation de cette page » demande une phrase.
        Field::new()
            .name(format!("spots.{index}.perk"))
            .label("i18n:host.spot.perk")
            .child(
                TextArea::new()
                    .name(format!("spots.{index}.perk"))
                    .value(perk)
                    .rows(2)
                    .placeholder("i18n:host.spot.perk.placeholder"),
            )
            .into(),
        Field::new()
            .name(format!("spots.{index}.note"))
            .label("i18n:host.spot.note")
            .child(
                TextArea::new()
                    .name(format!("spots.{index}.note"))
                    .value(note)
                    .rows(2)
                    .placeholder("i18n:host.spot.note.placeholder"),
            )
            .into(),
        FieldHint::new().text("i18n:host.spot.note.hint").into(),
        text_field(index, "price", "i18n:host.spot.price", price),
        text_field(index, "hours", "i18n:host.spot.hours", hours),
        text_field(index, "opening", "i18n:host.spot.opening", opening),
        text_field(index, "parking", "i18n:host.spot.parking", parking),
        text_field(index, "phone", "i18n:host.spot.phone", phone),
        text_field(index, "url", "i18n:host.spot.url", url),
        picker.into(),
        Field::new()
            .name(format!("spots.{index}.photo"))
            .label("i18n:host.spot.photo")
            .child(
                ImageUpload::new()
                    .name(format!("spots.{index}.photo"))
                    .value(photo),
            )
            .into(),
    ];

    Stack::new()
        .id(format!("spot-{index}"))
        .gap(10.0)
        .children(id.into_iter().chain(fields).collect())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_grow_with_the_list_and_stop_at_the_cap() {
        assert_eq!(activity_slots(0), 2);
        assert_eq!(activity_slots(1), 2);
        assert_eq!(activity_slots(4), 5);
        assert_eq!(activity_slots(MAX_CURATED_LINKS), MAX_CURATED_LINKS);
        // Le formulaire ne propose jamais un créneau que le livret n'afficherait pas.
        // Sauf une ligne déjà stockée au-delà : elle reste là où elle est.
        assert_eq!(activity_slots(MAX_CURATED_LINKS + 5), MAX_CURATED_LINKS + 5);
    }
}
