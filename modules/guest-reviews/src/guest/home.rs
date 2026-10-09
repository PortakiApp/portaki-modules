//! Guest home card — inline post-stay review (no overlay).
//!
//! The rating is always offered; the host's public review link (any platform) to every guest.

use portaki_sdk::host::i18n::{translate, Vars};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::ChoiceListLayout;
use portaki_sdk::sdui::primitives::{
    Button, Card, Celebration, ChoiceList, Divider, Field, Form, QRCode, Stack, Text, TextArea,
};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestData;

/// Le commentaire privé (spec §2.1) : la zone de texte après la note, quand l'hôte la propose.
fn comment_field(private_comment: bool) -> Vec<Component> {
    if !private_comment {
        return Vec::new();
    }
    vec![Field::new()
        .name("comment")
        .label("i18n:guest.comment")
        .child(
            TextArea::new()
                .name("comment")
                .placeholder("i18n:guest.commentPlaceholder"),
        )
        .into()]
}

pub fn build_home_card(data: &GuestData) -> Surface {
    let mut children = Vec::new();

    let prompt = t!("guest.prompt", property = data.property_name.as_str())
        .unwrap_or_else(|_| data.property_name.clone());
    children.push(Component::Text(
        Text::new().text(prompt).variant(TextVariant::Title),
    ));

    if !data.thank_you.is_empty() {
        children.push(Component::Text(
            Text::new()
                .text(data.thank_you.clone())
                .variant(TextVariant::Body),
        ));
    } else {
        children.push(Component::Text(
            Text::new()
                .text("i18n:guest.defaultThanks")
                .variant(TextVariant::Body),
        ));
    }

    // Le lien public, à tous, avant la note : filtrer selon la note est interdit (spec §3).
    if let Some((url, platform)) = &data.link {
        children.push(Component::Button(
            Button::new()
                .label(platform.cta_key())
                .action(Action::external(url.clone())),
        ));
    }

    // Déjà noté : le remerciement prend la place du formulaire. L'envoi refuse un second avis
    // (`review_already_submitted`) — le proposer quand même faisait remplir cinq étoiles et un
    // mot pour recevoir une erreur (§2.19).
    if let Some(rating) = data.rating_given {
        children.push(review_thanks(rating, &data.host_name));
    } else {
        if data.link.is_some() {
            children.push(Component::Text(
                Text::new()
                    .text("i18n:guest.orPortaki")
                    .variant(TextVariant::Caption),
            ));
        }

        let submit_action = crate::ids::module_id().command(
            crate::commands::SUBMIT_REVIEW,
            // Le livret remplace ces valeurs par celles du formulaire à l'envoi
            // (`mergeCommandFormValues`) ; `0` ne passe pas la validation de `submit_review`,
            // ce qui est exactement ce qu'il faut si le formulaire n'envoyait rien.
            crate::commands::SubmitReviewArgs {
                rating: 0,
                comment: String::new(),
            },
        );

        let form = Form::new().child(
            Field::new()
                .name("rating")
                .label("i18n:guest.rating")
                // Obligatoire, et aucune étoile préchoisie : cinq étoiles servies
                // d'avance partaient telles quelles chez qui touchait « Envoyer » sans
                // rien noter. Le livret retient l'envoi tant que rien n'est choisi.
                .required(true)
                // Des étoiles, pas une liste déroulante : une note se donne d'un doigt.
                // `Stars` dessine les cinq, et porte les flèches du clavier.
                .child(
                    ChoiceList::new()
                        .name("rating")
                        .layout(ChoiceListLayout::Stars)
                        .choices(vec![
                            ChoiceOption::new("1", "i18n:guest.rating.1"),
                            ChoiceOption::new("2", "i18n:guest.rating.2"),
                            ChoiceOption::new("3", "i18n:guest.rating.3"),
                            ChoiceOption::new("4", "i18n:guest.rating.4"),
                            ChoiceOption::new("5", "i18n:guest.rating.5"),
                        ]),
                ),
        );
        // `children` remplacerait la liste (la note avec) : le commentaire s'ajoute après elle.
        let form = comment_field(data.private_comment)
            .into_iter()
            .fold(form, |form, field| form.child(field));
        children.push(Component::Form(
            form.child(
                Button::new()
                    .label("i18n:guest.submit")
                    .action(submit_action),
            ),
        ));
    }

    // Le QR vient en dernier, séparé par un filet : il sert à finir l'avis sur un téléphone,
    // pas à le commencer (§2.19). Au-dessus du formulaire, il détournait du champ.
    if let Some((url, _)) = &data.link {
        if children.len() > 2 {
            children.push(Component::Divider(Divider::new()));
        }
        children.push(Component::QRCode(
            QRCode::new().value(url.clone()).size(144.0),
        ));
        children.push(Component::Text(
            Text::new()
                .text("i18n:guest.scanQr")
                .variant(TextVariant::Caption),
        ));
    }

    Surface::new(
        Card::new()
            .icon(IconName::Star)
            .title("i18n:home.card.title")
            .subtitle(thanks_line(&data.host_name))
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::guest::HOME_CARD)
}

/// « Merci pour votre avis · Claire le lira avec attention » (§2.19).
fn review_thanks(rating: u8, host_name: &str) -> Component {
    let message = if host_name.is_empty() {
        "i18n:guest.given.message".to_string()
    } else {
        let mut vars = Vars::new();
        vars.set("host", host_name);
        translate("guest.given.message.named", &vars)
            .unwrap_or_else(|_| "i18n:guest.given.message".to_string())
    };
    let mut vars = Vars::new();
    vars.set("rating", rating);
    Component::Celebration(
        Celebration::new()
            .emoji("💛")
            .title(
                translate("guest.given.title", &vars)
                    .unwrap_or_else(|_| "i18n:guest.given.title".to_string()),
            )
            .message(message),
    )
}

/// « Deux minutes, et Claire vous en remercie » — le nom de l'hôte vient de la plateforme.
///
/// Sans nom servi, la phrase se passe de lui plutôt que de laisser un trou ou un « votre hôte »
/// qui sonne comme un formulaire.
fn thanks_line(host_name: &str) -> String {
    if host_name.is_empty() {
        return translate("guest.thanks", &Vars::new())
            .unwrap_or_else(|_| "guest.thanks".to_string());
    }
    let mut vars = Vars::new();
    vars.set("host", host_name);
    translate("guest.thanks.named", &vars).unwrap_or_else(|_| "guest.thanks.named".to_string())
}
