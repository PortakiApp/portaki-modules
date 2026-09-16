//! Host dashboard surface — design `guide-editor-v1` (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Card, Field, Form, InfoBanner, Page, Stack, Text, TextArea, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::affiliate::{looks_like_url, normalize_curated_url, CuratedUrlError, MAX_CURATED_LINKS};
use crate::config::{load_config, ActivityRow, Localized, SpotRow};

const SPOT_SLOTS: usize = 6;

/// Créneaux d'activités affichés : ceux qui sont remplis, plus un libre.
///
/// Deux au minimum pour que la section ait l'air d'une liste, [`MAX_CURATED_LINKS`] au
/// maximum — la même borne que celle appliquée à l'enregistrement, pour que le formulaire
/// ne propose jamais un créneau que la commande refuserait.
fn activity_slots(filled: usize) -> usize {
    (filled + 1).clamp(2, MAX_CURATED_LINKS)
}

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = Localized::lang_code(&ctx.locale);
    let config = load_config().unwrap_or_default();
    let spots = config.parse_spots();
    let disclaimer = config.disclaimer.get(&lang).to_string();
    let activities = &config.activities;

    let activities_enabled = ctx.input_bool("activities_enabled", activities.enabled);
    let activities_destination = ctx
        .input_str("activities_destination")
        .map(str::to_string)
        .unwrap_or_else(|| activities.destination.clone());
    let activities_intro = ctx
        .input_str("activities_intro")
        .map(str::to_string)
        .unwrap_or_else(|| activities.intro.get(&lang).to_string());

    let submit_args = crate::commands::UpdateConfigArgs {
        spots: spots_to_submit(&spots, &lang),
        spots_json: String::new(),
        disclaimer: disclaimer.clone(),
        activities_enabled: Some(activities_enabled),
        activities_destination: activities_destination.clone(),
        activities_intro: activities_intro.clone(),
        activities: activities_to_submit(&activities.links, &lang),
    };
    let save_action = crate::ids::module_id().command(crate::ids::UPDATE_CONFIG, submit_args);

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
    cards.push(
        Card::new()
            .title("i18n:host.section.disclaimer")
            .icon("info-circle")
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
    cards.push(
        Button::new()
            .label("i18n:host.save")
            .tone(Tone::Primary)
            .action(save_action)
            .into(),
    );

    Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(vec![
                    Text::new()
                        .text("i18n:surface.host.main.subtitle")
                        .variant(TextVariant::Body)
                        .into(),
                    Component::Stack(Stack::new().gap(16.0).children(cards)),
                ]))),
    )
    .with_id(crate::ids::HOST_MAIN)
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
        .icon("ticket")
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
                    .map(|row| row.label.get(lang).to_string())
                    .unwrap_or_default()
            });

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
        .icon("ticket")
        .children(children)
        .into()
}

fn activities_to_submit(links: &[ActivityRow], lang: &str) -> Vec<crate::commands::ActivityInput> {
    links
        .iter()
        .map(|row| crate::commands::ActivityInput {
            url: row.url.clone(),
            label: row.label.get(lang).to_string(),
        })
        .collect()
}

fn spots_to_submit(spots: &[SpotRow], lang: &str) -> Vec<crate::commands::SpotInput> {
    spots
        .iter()
        .map(|s| crate::commands::SpotInput {
            name: s.title.get(lang).to_string(),
            category: s.category.clone().unwrap_or_default(),
            distance: s.distance.clone().unwrap_or_default(),
            tag: s.tag.clone().unwrap_or_default(),
            description: s
                .detail
                .as_ref()
                .map(|d| d.get(lang).to_string())
                .unwrap_or_default(),
        })
        .collect()
}

fn spot_card(index: usize, spot: Option<&SpotRow>, lang: &str) -> Component {
    let slot = index + 1;
    let name = spot.map(|s| s.title.get(lang)).unwrap_or("");
    let category = spot.and_then(|s| s.category.as_deref()).unwrap_or("");
    let distance = spot.and_then(|s| s.distance.as_deref()).unwrap_or("");
    let tag = spot.and_then(|s| s.tag.as_deref()).unwrap_or("");
    let description = spot
        .and_then(|s| s.detail.as_ref())
        .map(|d| d.get(lang))
        .unwrap_or("");

    Card::new()
        .title(format!("i18n:host.spot.slot{slot}"))
        .icon("map-pin")
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
        // Le formulaire ne propose jamais un créneau que `updateConfig` refuserait.
        assert_eq!(activity_slots(MAX_CURATED_LINKS + 5), MAX_CURATED_LINKS);
    }
}
