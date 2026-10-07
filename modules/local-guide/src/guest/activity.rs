//! La section et la fiche des activités que l'hôte propose lui-même (§2.13, `origin: host`).
//!
//! Elles ne viennent d'aucun fournisseur : l'hôte décrit ce qu'il connaît, la réservation se fait
//! avec le prestataire, et aucune commission n'est perçue. D'où une mention différente de celle
//! des liens partenaires, et un bouton qui appelle au lieu d'ouvrir une boutique.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{
    Author, BadgeSpec, Emphasis, KeyValueLayout, Leading, LeadingVisual, ListItemLayout,
    RichTextVariant, StackDirection, SurfaceLevel, Tone, Trailing, TrailingVisual,
};
use portaki_sdk::sdui::primitives::{
    Badge, Button, Card, Grid, Image, InfoBanner, KeyValue, ListItem, RichText, Stack, Text,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::HostActivityRow;

use super::load::GuestData;

/// La largeur d'une tuile de mesure (§2.13) : deux tiennent côte à côte sur un téléphone.
const TILE_WIDTH: f64 = 96.0;

/// « Les activités de Claire » : les tuiles qui défilent, et la mention sans commission.
///
/// Vide quand l'hôte n'en propose aucune — un titre seul annoncerait une section qui n'existe pas.
pub fn build_host_activities(data: &GuestData) -> Vec<Component> {
    if data.host_activities.is_empty() {
        return Vec::new();
    }

    let tiles: Vec<Component> = data
        .host_activities
        .iter()
        .enumerate()
        .map(|(index, activity)| tile(data, activity, index))
        .collect();

    vec![
        Component::Text(
            Text::new()
                .text(section_title(&data.host_name))
                .variant(TextVariant::Title),
        ),
        Component::Stack(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .scroll(true)
                .gap(8.0)
                .children(tiles),
        ),
        // La mention dit l'inverse de celle des partenaires, et c'est le point : ici personne ne
        // touche de commission, et le voyageur doit pouvoir le lire.
        Component::Text(
            Text::new()
                .text("i18n:guest.hostActivities.disclosure")
                .variant(TextVariant::Caption),
        ),
    ]
}

/// Une tuile : la photo, le nom, le prestataire et la durée, le prix en bout, « Choix de l'hôte ».
fn tile(data: &GuestData, activity: &HostActivityRow, index: usize) -> Component {
    let mut item = ListItem::new()
        .layout(ListItemLayout::Tile)
        .title(activity.title.get(&data.locale))
        .trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge: Some(BadgeSpec {
                label: choice_badge(&data.host_name),
                tone: Tone::Primary,
                ..BadgeSpec::default()
            }),
            ..TrailingVisual::default()
        })))
        .chevron(true)
        .action(Action::navigate(
            NavigateTarget::path(format!("local-guide/activity/{}", activity.route_id(index))),
            None,
        ));

    let subtitle = [activity.provider.as_deref(), activity.duration.as_deref()]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    if !subtitle.is_empty() {
        item = item.subtitle(subtitle);
    }
    // Un prix fixe, sans « dès » : l'hôte connaît le tarif de son prestataire, le fournisseur
    // partenaire annonce un plancher. Les deux ne se lisent pas pareil.
    if let Some(price) = activity.price.as_deref() {
        item = item.meta(price.to_string());
    }
    if let Some(reference) = activity.photo_ref() {
        item = item.leading(Leading::Visual(Box::new(LeadingVisual {
            image: Some(reference.to_string()),
            ..LeadingVisual::default()
        })));
    }
    Component::ListItem(item)
}

/// La fiche d'une activité de l'hôte.
pub fn build_activity_item(data: &GuestData, activity: &HostActivityRow) -> Surface {
    let mut children: Vec<Component> = Vec::new();

    if let Some(reference) = activity.photo_ref() {
        children.push(
            Image::new()
                .url(reference.to_string())
                .alt(activity.title.get(&data.locale))
                .aspectRatio("4 / 3")
                .into(),
        );
    }

    children.push(header(data, activity));

    if let Some(tiles) = measures(activity) {
        children.push(tiles);
    }

    // L'annulation en bandeau : c'est ce qu'on cherche avant de s'engager, et une ligne parmi
    // les infos pratiques se lisait après le reste.
    let cancel = activity.cancel.get(&data.locale).trim().to_string();
    if !cancel.is_empty() {
        children.push(InfoBanner::new().tone(Tone::Success).title(cancel).into());
    }

    if let Some(card) = included(data, activity) {
        children.push(card);
    }
    if let Some(card) = practical(data, activity) {
        children.push(card);
    }
    if let Some(card) = host_tip(data, activity) {
        children.push(card);
    }

    children.extend(actions(activity));
    children.push(
        Text::new()
            .text("i18n:guest.hostActivities.disclosure")
            .variant(TextVariant::Caption)
            .into(),
    );

    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_ACTIVITY)
}

/// « Choix de Claire », le nom, puis « Proposé par Claire · Marc, skipper au port ».
fn header(data: &GuestData, activity: &HostActivityRow) -> Component {
    let mut children: Vec<Component> = vec![
        Badge::new().label(choice_badge(&data.host_name)).into(),
        Text::new()
            .text(activity.title.get(&data.locale))
            .variant(TextVariant::Display)
            .into(),
    ];
    if let Some(provider) = activity.provider.as_deref() {
        children.push(
            Text::new()
                .text(offered_by(&data.host_name, provider))
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        );
    }
    Stack::new().gap(6.0).children(children).into()
}

/// Les tuiles : le prix et la durée, celles que l'hôte a données.
fn measures(activity: &HostActivityRow) -> Option<Component> {
    let mut tiles: Vec<Component> = Vec::new();
    if let Some(price) = activity.price.as_deref() {
        tiles.push(measure(
            IconName::Star,
            "i18n:guest.activity.price",
            price.to_string(),
        ));
    }
    if let Some(duration) = activity.duration.as_deref() {
        tiles.push(measure(
            IconName::Clock,
            "i18n:guest.activity.duration",
            duration.to_string(),
        ));
    }
    (!tiles.is_empty()).then(|| {
        Grid::new()
            .minColumnWidth(TILE_WIDTH)
            .plain(true)
            .children(tiles)
            .into()
    })
}

fn measure(icon: IconName, label: &str, value: String) -> Component {
    KeyValue::new()
        .layout(KeyValueLayout::Tile)
        .icon(icon)
        .key(label)
        .value(value)
        .into()
}

/// « Inclus » : une ligne par élément, cochée.
fn included(data: &GuestData, activity: &HostActivityRow) -> Option<Component> {
    let lines = activity.included_lines(&data.locale);
    (!lines.is_empty()).then(|| {
        Component::Card(
            Card::new()
                .surface(SurfaceLevel::Elevated)
                .icon(IconName::CheckCircle)
                .title("i18n:guest.activity.included")
                .children(
                    lines
                        .into_iter()
                        .map(|line| {
                            ListItem::new()
                                .title(line)
                                .leading(Leading::Icon("check".into()))
                                .into()
                        })
                        .collect::<Vec<Component>>(),
                ),
        )
    })
}

/// « Infos pratiques » : le rendez-vous, les langues, et de quoi joindre le prestataire.
fn practical(data: &GuestData, activity: &HostActivityRow) -> Option<Component> {
    let _ = data;
    let mut rows: Vec<Component> = Vec::new();
    if let Some(meet) = activity.meet.as_deref() {
        rows.push(row("i18n:guest.activity.meet", meet, false));
    }
    if let Some(languages) = activity.languages.as_deref() {
        rows.push(row("i18n:guest.activity.languages", languages, false));
    }
    if let Some(phone) = activity.phone.as_deref() {
        rows.push(row("i18n:guest.activity.phone", phone, true));
    }
    (!rows.is_empty()).then(|| {
        Component::Card(
            Card::new()
                .surface(SurfaceLevel::Elevated)
                .icon(IconName::InfoCircle)
                .title("i18n:guest.practical.title")
                .children(rows),
        )
    })
}

fn row(key: &str, value: &str, mono: bool) -> Component {
    let mut node = KeyValue::new().key(key).value(value.to_string());
    if mono {
        node = node.mono(true);
    }
    node.into()
}

/// « Le conseil de Claire » : ce qu'aucun fournisseur n'écrira.
///
/// Citation signée de §8 : le module marque le bloc avec `author`, la coquille y met l'hôte du
/// profil. Rien ici ne nomme le prestataire de l'activité — le conseil est de l'hôte, pas de lui.
fn host_tip(data: &GuestData, activity: &HostActivityRow) -> Option<Component> {
    let tip = activity.tip.get(&data.locale).trim().to_string();
    if tip.is_empty() {
        return None;
    }
    Some(Component::Card(
        Card::new()
            .surface(SurfaceLevel::Elevated)
            .icon(IconName::Message)
            .title(tip_title(&data.host_name))
            .child(
                RichText::new()
                    .content(tip)
                    .variant(RichTextVariant::Lead)
                    .author(Author::default()),
            ),
    ))
}

/// « Réserver auprès de Marc » — un appel, pas une boutique. Puis le site, s'il en a un.
fn actions(activity: &HostActivityRow) -> Vec<Component> {
    let mut bar: Vec<Component> = Vec::new();
    if let Some(phone) = activity.phone.as_deref() {
        bar.push(
            Button::new()
                .label(book_label(activity.provider.as_deref()))
                .action(Action::external(tel_url(phone)))
                .into(),
        );
    }
    if let Some(url) = activity.url.as_deref() {
        let mut button = Button::new()
            .label("i18n:guest.activity.website")
            .action(Action::external(url.to_string()));
        if !bar.is_empty() {
            button = button.variant(ButtonVariant::Outline);
        }
        bar.push(button.into());
    }
    bar
}

/// `tel:` sans espace ni séparateur : « +33 6 22 33 44 55 » tel quel ne se compose pas.
fn tel_url(phone: &str) -> String {
    let digits: String = phone
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();
    format!("tel:{digits}")
}

fn section_title(host_name: &str) -> String {
    if host_name.is_empty() {
        return "i18n:guest.hostActivities.title".to_string();
    }
    t!(
        "guest.hostActivities.title.named",
        host = host_name.to_string()
    )
    .unwrap_or_else(|_| "i18n:guest.hostActivities.title".into())
}

fn choice_badge(host_name: &str) -> String {
    if host_name.is_empty() {
        return "i18n:guest.hostActivities.choice".to_string();
    }
    t!(
        "guest.hostActivities.choice.named",
        host = host_name.to_string()
    )
    .unwrap_or_else(|_| "i18n:guest.hostActivities.choice".into())
}

fn offered_by(host_name: &str, provider: &str) -> String {
    if host_name.is_empty() {
        return t!("guest.activity.offeredBy", provider = provider.to_string())
            .unwrap_or_else(|_| provider.to_string());
    }
    t!(
        "guest.activity.offeredBy.named",
        host = host_name.to_string(),
        provider = provider.to_string()
    )
    .unwrap_or_else(|_| provider.to_string())
}

fn tip_title(host_name: &str) -> String {
    if host_name.is_empty() {
        return t!("guest.tip.title").unwrap_or_else(|_| "i18n:guest.tip.title".into());
    }
    t!("guest.tip.title.named", host = host_name.to_string())
        .unwrap_or_else(|_| "i18n:guest.tip.title".into())
}

fn book_label(provider: Option<&str>) -> String {
    match provider {
        Some(provider) => t!("guest.activity.book", provider = provider.to_string())
            .unwrap_or_else(|_| "i18n:guest.activity.book.plain".into()),
        None => "i18n:guest.activity.book.plain".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::tel_url;

    #[test]
    fn a_number_dials_without_its_spaces() {
        assert_eq!(tel_url("+33 6 22 33 44 55"), "tel:+33622334455");
    }
}
