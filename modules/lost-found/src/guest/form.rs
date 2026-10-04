//! Guest bottom-sheet form surface opened from the home card.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    ChoiceList, Field, FieldHint, Form, ImageUpload, InfoBanner, Stack, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

/// Les catégories d'objet de la maquette (§2.18), avec l'emoji qui les fait reconnaître d'un
/// coup d'œil. Une liste figée : un voyageur qui cherche son chargeur ne doit pas inventer un mot
/// pour le ranger, et l'hôte lit la même étiquette que lui.
const CATEGORIES: [(&str, &str); 8] = [
    ("phone", "📱"),
    ("clothing", "👕"),
    ("glasses", "🕶️"),
    ("jewellery", "💍"),
    ("papers", "🔑"),
    ("toy", "🧸"),
    ("toiletries", "🧴"),
    ("other", ""),
];

/// Les pièces de la maquette, `unknown` compris : « Je ne sais pas » est une réponse, et la forcer
/// à choisir une pièce ferait désigner la chambre au hasard.
const ROOMS: [&str; 6] = [
    "bedroom", "bathroom", "kitchen", "living", "outside", "unknown",
];

/// Bottom-sheet lost/found form (inputs live here — not on the home card).
#[portaki_sdk::surface(
    guest,
    id = "guest.form",
    path = "lost-found/form",
    label_key = "nav.lost-found"
)]
pub fn render_guest_form(ctx: GuestContext) -> Result<Surface> {
    let config = crate::config::ModuleConfig::load(&ctx)?;
    Ok(build_form_surface(
        config.offers_shipping(),
        config.return_options(),
        deadline_label(&ctx, config.window_days()),
        ctx.host
            .as_ref()
            .map(|host| host.name.trim().to_string())
            .filter(|name| !name.is_empty()),
    ))
}

pub fn build_form_surface(
    ask_address: bool,
    return_options: Vec<&'static str>,
    deadline: Option<String>,
    host_name: Option<String>,
) -> Surface {
    Surface::new(build_form(ask_address, return_options, deadline, host_name)).with_id(GUEST_FORM)
}

/// Le formulaire en trois étapes du §2.18.
///
/// `Form::wizard` + un `Stack::step` par étape : le livret dessine la barre de progression, le
/// titre de l'étape, « Retour » et « Continuer », et n'affiche que l'étape courante. D'où
/// `submitLabel` au lieu d'un `Button` enfant — un bouton enfant deviendrait une quatrième étape.
///
/// Trois questions, trois écrans : quel objet, où il est resté, ce qu'on en fait. En une seule
/// page, le choix de restitution — la question qui engage — se lisait après dix champs.
fn build_form(
    ask_address: bool,
    return_options: Vec<&'static str>,
    deadline: Option<String>,
    host_name: Option<String>,
) -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::commands::SUBMIT);

    // Les enfants sont assemblés puis posés d'un coup : `children` remplace la liste, il ne
    // l'allonge pas, et l'appeler au milieu d'une chaîne de `child` efface ce qui précède.
    let object_step: Vec<Component> = vec![
        Field::new()
            .name("kind")
            .label("i18n:form.kind.label")
            .required(true)
            .child(kind_choice_list())
            .into(),
        // Quel objet, en tuiles : la grille se touche d'un doigt là où une liste déroulante
        // demande de lire huit libellés avant d'en choisir un (§2.18).
        Field::new()
            .name("category")
            .label("i18n:form.category.label")
            .child(category_choice_list())
            .into(),
        Field::new()
            .name("itemDescription")
            .label("i18n:form.itemDescription.label")
            .required(true)
            .child(
                TextInput::new()
                    .name("itemDescription")
                    .placeholder("i18n:form.itemDescription.placeholder"),
            )
            .into(),
        FieldHint::new()
            .text("i18n:form.itemDescription.hint")
            .into(),
    ];

    let where_step: Vec<Component> = vec![
        Field::new()
            .name("room")
            .label("i18n:form.room.label")
            .child(room_choice_list())
            .into(),
        Field::new()
            .name("contactHint")
            .label("i18n:form.contactHint.label")
            .child(
                TextInput::new()
                    .name("contactHint")
                    .placeholder("i18n:form.contactHint.placeholder"),
            )
            .into(),
        Field::new()
            .name("details")
            .label("i18n:form.details.label")
            .child(
                TextArea::new()
                    .name("details")
                    .placeholder("i18n:form.details.placeholder"),
            )
            .into(),
        // Une photo vaut toute la description : l'hôte reconnaît le chargeur sur la table de nuit
        // sans avoir à deviner ce que « blanc, petit » veut dire.
        Field::new()
            .name("photo")
            .label("i18n:form.photo.label")
            .child(ImageUpload::new().name("photo"))
            .into(),
    ];

    let mut return_step: Vec<Component> = Vec::new();
    return_step.extend(return_choice_field(&return_options));
    return_step.extend(address_field(ask_address));
    // Sans renvoi, le dire : un voyageur qui attend un colis et n'en reçoit pas se demande ce
    // qu'il a mal rempli (§2.18).
    if !ask_address {
        return_step.push(
            InfoBanner::new()
                .tone(Tone::Neutral)
                .message(no_shipping_message(host_name.as_deref()))
                .into(),
        );
    }
    // Le délai en dernier : il ne presse pas celui qui remplit, il rassure celui qui hésite.
    if let Some(deadline) = deadline {
        return_step.push(
            InfoBanner::new()
                .tone(Tone::Neutral)
                .title(deadline)
                .message("i18n:form.deadline.message")
                .into(),
        );
    }

    Form::new()
        .wizard(true)
        .submitLabel("i18n:form.submit")
        .onSubmit(submit_action)
        .children(vec![
            Component::Stack(
                Stack::new()
                    .step("i18n:form.step.object")
                    .gap(12.0)
                    .children(object_step),
            ),
            Component::Stack(
                Stack::new()
                    .step("i18n:form.step.where")
                    .gap(12.0)
                    .children(where_step),
            ),
            Component::Stack(
                Stack::new()
                    .step(step_return_label(host_name.as_deref()))
                    .gap(12.0)
                    .children(return_step),
            ),
        ])
}

/// « Si Claire le retrouve », ou « Si votre hôte le retrouve » sans prénom servi.
fn step_return_label(host_name: Option<&str>) -> String {
    match host_name {
        Some(host) => t!("form.step.found.named", host = host.to_string())
            .unwrap_or_else(|_| "i18n:form.step.found".into()),
        None => "i18n:form.step.found".to_string(),
    }
}

/// « Claire ne renvoie pas les objets oubliés : elle le garde jusqu'à votre passage. »
fn no_shipping_message(host_name: Option<&str>) -> String {
    match host_name {
        Some(host) => t!("form.noShipping.named", host = host.to_string())
            .unwrap_or_else(|_| "i18n:form.noShipping".into()),
        None => "i18n:form.noShipping".to_string(),
    }
}

/// Ce que le voyageur voudrait qu'on en fasse, parmi ce que l'hôte propose.
///
/// Rien à choisir quand l'hôte ne propose qu'une option : la question aurait une seule réponse,
/// et un choix à un terme se lit comme une case à cocher obligatoire.
fn return_choice_field(return_options: &[&'static str]) -> Vec<Component> {
    if return_options.len() < 2 {
        return Vec::new();
    }
    let choices: Vec<ChoiceOption> = return_options
        .iter()
        .map(|option| {
            ChoiceOption::new(*option, format!("i18n:form.return.{option}"))
                .description(format!("i18n:form.return.{option}.description"))
                .icon(return_icon(option))
        })
        .collect();
    vec![
        Field::new()
            .name("returnChoice")
            .label("i18n:form.return.label")
            // L'étape existe pour cette question : la quitter sans y répondre laisse l'hôte sans
            // savoir quoi faire de l'objet qu'il vient de retrouver (§2.18).
            .required(true)
            .child(
                ChoiceList::new()
                    .name("returnChoice")
                    .layout(ChoiceListLayout::List)
                    .choices(choices),
            )
            .into(),
        FieldHint::new().text("i18n:form.return.hint").into(),
    ]
}

fn return_icon(option: &str) -> IconName {
    match option {
        "pickup" => IconName::Home,
        "donate" => IconName::Sparkles,
        _ => IconName::Send,
    }
}

/// « Jusqu'au 5 septembre » — la date de la fin du délai, écrite pour le voyageur.
///
/// Rien sans date de départ ni horloge : le délai ne court pas davantage, et annoncer une échéance
/// qu'on ne sait pas calculer vaut moins que ne rien annoncer.
fn deadline_label(ctx: &GuestContext, window_days: u32) -> Option<String> {
    let checkout_at = ctx.stay.as_ref().and_then(|stay| stay.checkout_at)?;
    let deadline = checkout_at + chrono::Duration::days(i64::from(window_days));
    // Dans le fuseau du logement : un départ le 8 à 10 h plus sept jours ne tombe pas la veille
    // parce que le voyageur lit son livret depuis Tokyo.
    let local = match portaki_sdk::host::time::PropertyTz::parse(&ctx.timezone) {
        Some(tz) => tz.to_local(deadline).date_naive(),
        None => deadline.date_naive(),
    };
    let formatted = portaki_sdk::host::time::long_date(local, &ctx.locale);
    t!("form.deadline.title", date = formatted).ok()
}

/// L'adresse de renvoi, seulement quand l'hôte propose le renvoi (§10).
///
/// La demander sans proposer le renvoi ferait écrire son adresse à un voyageur pour rien — et une
/// adresse postale est une donnée qu'on ne collecte pas « au cas où ».
fn address_field(ask_address: bool) -> Vec<Component> {
    if !ask_address {
        return Vec::new();
    }
    // ponytail: l'adresse reste facultative et visible dès que l'hôte propose le renvoi, là où
    // §2.18 la veut obligatoire et révélée par « Me le renvoyer ». L'exiger sans la révéler
    // forcerait une adresse à qui choisit le passage ; la révéler demande une visibilité
    // conditionnelle que les primitives n'ont pas (`Field.visibleWhen`).
    vec![Field::new()
        .name("returnAddress")
        .label("i18n:form.returnAddress.label")
        .child(
            TextArea::new()
                .name("returnAddress")
                .placeholder("i18n:form.returnAddress.placeholder"),
        )
        .into()]
}

fn kind_choice_list() -> ChoiceList {
    ChoiceList::new()
        .name("kind")
        .layout(ChoiceListLayout::Compact)
        .choices(vec![
            ChoiceOption::new("lost", "i18n:form.kind.lost").icon(IconName::SearchX),
            ChoiceOption::new("found", "i18n:form.kind.found").icon(IconName::PackageSearch),
        ])
}

fn category_choice_list() -> ChoiceList {
    let choices: Vec<ChoiceOption> = CATEGORIES
        .iter()
        .map(|(value, emoji)| {
            let option = ChoiceOption::new(*value, format!("i18n:form.category.{value}"));
            if emoji.is_empty() {
                // « Autre » n'a pas d'objet à dessiner : un signe, et non un emoji choisi au
                // hasard parmi ceux qui restent.
                option.icon(IconName::Plus)
            } else {
                option.emoji(*emoji)
            }
        })
        .collect();
    ChoiceList::new()
        .name("category")
        .layout(ChoiceListLayout::Grid)
        .choices(choices)
}

fn room_choice_list() -> ChoiceList {
    ChoiceList::new()
        .name("room")
        .layout(ChoiceListLayout::Compact)
        .choices(
            ROOMS
                .iter()
                .map(|room| ChoiceOption::new(*room, format!("i18n:form.room.{room}")))
                .collect(),
        )
}
