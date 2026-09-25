//! Host dashboard surface — design `guide-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, Field, Form, InfoBanner, Page, Select, Stack, Text, TextArea,
    TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::affiliate::{looks_like_url, normalize_curated_url, CuratedUrlError, MAX_CURATED_LINKS};
use crate::config::{ActivityRow, Localized, ModuleConfig, SpotRow, TIQETS_RADIUS_CHOICES_KM};
use crate::tiqets::TiqetsStatus;

const SPOT_SLOTS: usize = 6;

/// Créneaux d'activités affichés : ceux qui sont remplis, plus un libre.
///
/// Deux au minimum pour que la section ait l'air d'une liste, [`MAX_CURATED_LINKS`] au
/// maximum — la même borne que celle appliquée au rendu voyageur, pour que le formulaire
/// ne propose jamais un créneau qui ne s'afficherait pas.
fn activity_slots(filled: usize) -> usize {
    (filled + 1).clamp(2, MAX_CURATED_LINKS)
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
    let lang = Localized::lang_code(&ctx.locale);
    let config = ModuleConfig::read(&ctx)?;
    let spots = config.parse_spots();
    // Un texte par champ désormais (la plateforme garde ce qu'envoie le formulaire) : on montre
    // la langue qui l'a, plutôt qu'un champ vide que le prochain enregistrement réécrirait.
    let disclaimer = config.disclaimer.pick(&lang);
    let activities = config.activities();
    let tiqets = config.tiqets();

    let activities_enabled = ctx.input_bool("activities_enabled", activities.enabled);
    let activities_destination = ctx
        .input_str("activities_destination")
        .map(str::to_string)
        .unwrap_or_else(|| activities.destination.clone());
    let activities_intro = ctx
        .input_str("activities_intro")
        .map(str::to_string)
        .unwrap_or_else(|| activities.intro.pick(&lang));

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

    let mut cards: Vec<Component> = Vec::new();
    for index in 0..SPOT_SLOTS {
        cards.push(spot_card(index, spots.get(index), &lang));
    }
    cards.push(activities_card(
        &ctx,
        activities_enabled,
        &activities_destination,
        &activities_intro,
        &activities.links,
        &lang,
    ));
    cards.push(tiqets_card(
        tiqets_enabled,
        &tiqets_radius,
        &tiqets_min_rating,
        tiqets_status,
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
    .with_id(crate::ids::HOST_MAIN))
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
    lang: &str,
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
            .unwrap_or_else(|| stored.map(|row| row.label.pick(lang)).unwrap_or_default());

        if matches!(
            normalize_curated_url(&url),
            Err(CuratedUrlError::NotGetYourGuide)
        ) {
            has_rejected_url = true;
        }

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

fn spot_card(index: usize, spot: Option<&SpotRow>, lang: &str) -> Component {
    let slot = index + 1;
    let name = spot.map(|s| s.title.pick(lang)).unwrap_or_default();
    let category = spot.and_then(|s| s.category.as_deref()).unwrap_or("");
    let distance = spot.and_then(|s| s.distance.as_deref()).unwrap_or("");
    let tag = spot.and_then(|s| s.tag.as_deref()).unwrap_or("");
    let description = spot
        .and_then(|s| s.detail.as_ref())
        .map(|d| d.pick(lang))
        .unwrap_or_default();
    let address = spot.and_then(|s| s.address.as_deref()).unwrap_or("");
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

    Card::new()
        .title(format!("i18n:host.spot.slot{slot}"))
        .icon(IconName::MapPin)
        .children(vec![
            Field::new()
                .name(format!("spots.{index}.name"))
                .label("i18n:host.spot.name")
                .child(
                    TextInput::new()
                        .name(format!("spots.{index}.name"))
                        .value(name),
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
                .name(format!("spots.{index}.description"))
                .label("i18n:host.spot.description")
                .child(
                    TextArea::new()
                        .name(format!("spots.{index}.description"))
                        .value(description),
                )
                .into(),
            picker.into(),
        ])
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
        assert_eq!(activity_slots(MAX_CURATED_LINKS + 5), MAX_CURATED_LINKS);
    }
}
