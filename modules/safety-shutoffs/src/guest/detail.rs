//! Guest fullscreen detail — le bandeau du 112, la consigne de l'hôte, un bloc par organe.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::NavigateTarget;
use portaki_sdk::sdui::common::{Emphasis, SurfaceLevel};
use portaki_sdk::sdui::primitives::{Button, Card, Image, InfoBanner, Stack, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, ShutoffRow};

/// La page de la section Aide, où « Numéros d'urgence » renvoie.
const AIDE: &str = "aide";

pub fn build_detail_surface(config: &ModuleConfig, ctx: &GuestContext) -> Surface {
    let mut children: Vec<Component> = vec![header(), danger_banner()];

    // La consigne de l'hôte, s'il en a écrit une : un bandeau d'information qui n'informe de rien
    // prend la place du seul bandeau qui compte, juste au-dessus.
    if !config.general_note.is_blank() {
        children.push(
            InfoBanner::new()
                .tone(Tone::Info)
                .title("i18n:guest.note.title")
                .message(config.general_note.for_ctx(ctx).to_string())
                .into(),
        );
    }

    children.extend(
        config
            .parse_shutoffs()
            .iter()
            .map(|row| shutoff_card(row, ctx)),
    );
    children.extend(bottom_bar(ctx));

    Surface::new(Stack::new().gap(14.0).children(children)).with_id(crate::guest::EXPLORE_DETAIL)
}

fn header() -> Component {
    Stack::new()
        .gap(4.0)
        .children(vec![
            Text::new()
                .text("i18n:guest.title")
                .variant(TextVariant::Title)
                .into(),
            Text::new()
                .text("i18n:guest.subtitle")
                .variant(TextVariant::Caption)
                .emphasis(Emphasis::Subtle)
                .into(),
        ])
        .into()
}

/// Le seul bloc de ton danger du livret (§1.9), et il ne vient pas de l'hôte.
///
/// Fixe : une odeur de gaz ne se traite pas en coupant une vanne, et un hôte qui écrirait sa propre
/// consigne ici remplacerait le 112 par un numéro qui sonne dans le vide. Pas d'`EmergencyButton`
/// non plus — le registre rouge reste au 112, appeler l'hôte est un bouton normal.
fn danger_banner() -> Component {
    InfoBanner::new()
        .tone(Tone::Danger)
        .title("i18n:guest.danger.title")
        .message("i18n:guest.danger.message")
        .into()
}

/// Un organe : son glyphe, son titre, son emplacement en gras, sa consigne en légende.
///
/// L'emplacement passe avant la consigne et en gras : devant un tableau électrique, ce qu'on
/// cherche d'abord est l'endroit, pas le geste. La photo attend le lot de stockage de fichiers
/// (`core.images` n'a pas d'API hôte dans le SDK) — v1 sans photo, cas « Coupures · sans photos ».
fn shutoff_card(row: &ShutoffRow, ctx: &GuestContext) -> Component {
    let mut children: Vec<Component> = vec![Text::new()
        .text(row.location.for_ctx(ctx).to_string())
        .variant(TextVariant::Body)
        .emphasis(Emphasis::Strong)
        .into()];
    // La photo entre l'endroit et la consigne, comme la maquette l'ordonne : on lit où, on voit
    // quoi, on lit comment. La référence devient une URL signée au rendu.
    if let Some(reference) = row.photo_ref() {
        children.push(
            Image::new()
                .url(reference.to_string())
                .alt(row.title.for_ctx(ctx).to_string())
                .aspectRatio("4 / 3")
                .into(),
        );
    }
    if !row.instruction.is_blank() {
        children.push(
            Text::new()
                .text(row.instruction.for_ctx(ctx).to_string())
                .variant(TextVariant::Caption)
                .into(),
        );
    }

    Card::new()
        .surface(SurfaceLevel::Elevated)
        .icon(row.icon())
        .title(row.title.for_ctx(ctx).to_string())
        .children(children)
        .into()
}

/// « Appeler {hôte} » puis « Numéros d'urgence ».
///
/// Le téléphone vient du profil de l'hôte, jamais d'un champ de ce module : un numéro saisi deux
/// fois finit par différer, et le livret en a déjà un (`emergency-contacts`, le socle du livret).
/// Sans numéro, pas de bouton — c'est ce que le SDK prescrit pour `HostProfile::phone`, et c'est
/// l'état d'aujourd'hui : le runtime ne sert pas encore `context.host`.
fn bottom_bar(ctx: &GuestContext) -> Vec<Component> {
    let mut bar: Vec<Component> = Vec::new();
    if let Some(phone) = ctx
        .host
        .as_ref()
        .and_then(|host| host.phone.as_deref())
        .map(str::trim)
        .filter(|phone| !phone.is_empty())
    {
        let label = ctx
            .host
            .as_ref()
            .map(|host| host.name.trim())
            .filter(|name| !name.is_empty())
            .and_then(|name| t!("guest.call", host = name).ok())
            .unwrap_or_else(|| "i18n:guest.call.plain".to_string());
        bar.push(
            Button::new()
                .label(label)
                .action(Action::external(format!("tel:{phone}")))
                .into(),
        );
    }
    bar.push(
        Button::new()
            .label("i18n:guest.emergency")
            .variant(ButtonVariant::Outline)
            .action(Action::navigate(NavigateTarget::path(AIDE), None))
            .into(),
    );
    bar
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_sdk::context::HostProfile;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    /// Le bouton d'appel, quand le profil de l'hôte porte un numéro.
    ///
    /// Le constructeur de mock ne sait pas poser `context.host` — le runtime ne le sert pas encore
    /// — donc le test le pose lui-même. C'est la seule façon de couvrir la branche sans attendre
    /// la plateforme, et le jour où elle arrive il n'y aura rien à changer ici.
    #[test]
    fn a_host_with_a_phone_gets_a_call_button() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "shutoffs": [{ "kind": "water", "title": "Vanne", "location": "Cave" }]
        }))
        .expect("config");

        MockContext::guest().run(|mut ctx| {
            ctx.host = Some(HostProfile {
                name: "Claire".into(),
                phone: Some("+33612345678".into()),
                ..HostProfile::default()
            });
            let json = serde_json::to_string(&build_detail_surface(&config, &ctx)).expect("json");
            assert!(json.contains("tel:+33612345678"), "{json}");
            // Le second bouton reste, et il ne duplique pas l'appel.
            assert!(json.contains("guest.emergency"), "{json}");

            // Un profil sans numéro ne dessine pas de bouton qui échoue.
            ctx.host = Some(HostProfile {
                name: "Claire".into(),
                phone: None,
                ..HostProfile::default()
            });
            let json = serde_json::to_string(&build_detail_surface(&config, &ctx)).expect("json");
            assert!(!json.contains("tel:"), "{json}");
        });
    }
}
