//! La fiche d'un appareil : consigne de sécurité, mode d'emploi, notices, et « un souci ? ».

use portaki_sdk::prelude::*;
use portaki_sdk::resource::rich_text;
use portaki_sdk::sdui::common::{Emphasis, Leading, LeadingVisual, SurfaceLevel};
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Eyebrow, Icon, Image, InfoBanner, ListItem, RichText, Stack, Text,
};

use super::no_appliances;
use crate::domain::{extract_howto_steps, is_video_url, Appliance, Guide, HowToStep};

/// Ce que « Un souci avec cet appareil ? » passe au formulaire de signalement : les noms que lit
/// le contrat `report.issue`. Pas de mode d'emploi ici — il finirait dans une URL.
#[portaki_sdk::wire(serialize)]
struct ReportIssueParams<'a> {
    category: &'static str,
    summary: &'a str,
}

/// `deviceId` arrive par les paramètres de route.
#[portaki_sdk::surface(
    guest,
    id = "explore.item",
    path = "appliances/:deviceId",
    label_key = "nav.appliance"
)]
pub fn render_explore_item(ctx: GuestContext) -> Result<Surface> {
    let guide = Guide::load(&ctx)?;
    if guide.is_empty() {
        return Ok(no_appliances(EXPLORE_ITEM));
    }
    let id = ctx.input.get("deviceId").and_then(|v| v.as_str());
    let Some(device) = id.and_then(|id| guide.find(id)) else {
        let missing = EmptyState::new()
            .title("i18n:explore.item.notFound")
            .description("i18n:explore.item.notFound.description")
            .icon(IconName::Plug);
        return Ok(Surface::new(Stack::new().child(missing)).with_id(EXPLORE_ITEM));
    };
    let children = detail(
        device,
        &guide.paper_manuals_location,
        report_path().as_deref(),
    );
    Ok(Surface::new(Stack::new().gap(14.0).children(children)).with_id(EXPLORE_ITEM))
}

/// Le chemin du formulaire de signalement, quand un module installé le fournit ; une erreur de la
/// plateforme coûte le bouton, pas la fiche.
fn report_path() -> Option<String> {
    portaki_sdk::host::module::list_by_capability(crate::REPORT_ISSUE)
        .ok()?
        .into_iter()
        .find_map(|peer| peer.path)
}

fn detail(device: &Appliance, paper: &str, report_path: Option<&str>) -> Vec<Component> {
    let mut children = vec![header(device)];
    // La consigne de sécurité en tête de fiche, avant le mode d'emploi (§2.2).
    if !device.safety_note.trim().is_empty() {
        children.push(InfoBanner::new().message(device.safety_note.clone()).into());
    }
    children.extend(howto(device));
    // Ni étapes, ni texte, ni notice : la fiche le dit, le bouton souci reste dessous (§9 #4).
    if !device.has_manual() {
        let none = Text::new().text("i18n:explore.item.noManual");
        children.push(none.variant(TextVariant::Caption).into());
    }
    children.extend(manuals_card(device, paper));
    if let Some(path) = report_path {
        let report = Action::navigate(
            NavigateTarget::path(path),
            Some(json_value(ReportIssueParams {
                category: "appliance",
                summary: &device.name,
            })),
        );
        let button = Button::new()
            .label("i18n:explore.item.contactHost")
            .variant(ButtonVariant::Outline)
            .action(report);
        children.push(button.into());
    }
    children
}

/// « Mode d'emploi » : les étapes saisies, puis le texte libre sous elles ; sans étapes, celles
/// des listes du texte libre ; sans liste, le texte tel quel. Le document TipTap part tel quel :
/// le livret le convertit, et refuse d'injecter autre chose.
fn howto(device: &Appliance) -> Option<Component> {
    let description = device.description.trim();
    let has_text = !rich_text::plain_text(description).trim().is_empty();
    let structured = !device.steps.is_empty();
    let steps = if structured {
        device.steps.clone()
    } else {
        extract_howto_steps(description)
    };
    let text = || RichText::new().content(description.to_string()).into();
    let mut children: Vec<Component> = vec![Eyebrow::new().text("i18n:explore.item.howto").into()];
    if steps.is_empty() {
        if !has_text {
            return None;
        }
        children.push(text());
    } else {
        children.extend(
            steps
                .iter()
                .enumerate()
                .map(|(i, step)| step_row(i + 1, step)),
        );
        if structured && has_text {
            children.push(text());
        }
    }
    let card = Card::new().surface(SurfaceLevel::Elevated);
    Some(card.children(children).into())
}

/// Le numéro en tête de rangée, la consigne en titre, le schéma dessous.
fn step_row(rank: usize, step: &HowToStep) -> Component {
    let row = ListItem::new()
        .title(step.text.clone())
        .leading(Leading::Visual(Box::new(LeadingVisual {
            index: Some(rank as u32),
            ..LeadingVisual::default()
        })));
    match step.image.as_deref() {
        Some(image) => {
            let alt = t!(
                "explore.item.step.alt",
                rank = rank,
                step = step.text.as_str()
            )
            .unwrap_or_else(|_| format!("{rank}. {}", step.text));
            let picture = Image::new()
                .url(image.to_string())
                .alt(alt)
                .aspectRatio("4 / 3");
            Stack::new().gap(6.0).child(row).child(picture).into()
        }
        None => row.into(),
    }
}

/// « Notices » : la vidéo, le PDF (à la place du lien quand les deux existent), le lien, et où
/// trouver la version papier (§3.1). Rien de tout cela : pas de carte.
fn manuals_card(device: &Appliance, paper: &str) -> Option<Component> {
    let pdf = device.manual_file.trim();
    let url = if pdf.is_empty() {
        device.manual_url.trim()
    } else {
        ""
    };
    let paper = paper.trim();
    let video = device.video_url.trim();
    let video = (video.starts_with("https://") && is_video_url(video)).then_some(video);
    let link = |title: &str, icon: &str, to: &str| -> Component {
        ListItem::new()
            .title(title)
            .leading(Leading::Icon(icon.into()))
            .chevron(true)
            .action(Action::external(to.to_string()))
            .into()
    };
    let mut rows: Vec<Component> = Vec::new();
    rows.extend(video.map(|v| link("i18n:explore.item.video", "link", v)));
    if !pdf.is_empty() {
        rows.push(link("i18n:explore.item.manual.pdf", "file-text", pdf));
    }
    if !url.is_empty() {
        rows.push(link("i18n:explore.item.manual", "file-text", url));
    }
    if !paper.is_empty() {
        let row = ListItem::new()
            .title("i18n:explore.item.manual.paper")
            .subtitle(paper.to_string())
            .leading(Leading::Icon("home".into()));
        rows.push(row.into());
    }
    if rows.is_empty() {
        return None;
    }
    let card = Card::new()
        .surface(SurfaceLevel::Elevated)
        .icon(IconName::FileText)
        .title("i18n:explore.item.manuals");
    Some(card.children(rows).into())
}

/// L'emoji en pictogramme (64 px), le nom en titre, « modèle · pièce » dessous.
fn header(device: &Appliance) -> Component {
    let name = Text::new()
        .text(device.name.clone())
        .variant(TextVariant::Display);
    let mut title = Stack::new().gap(4.0).child(name);
    let caption: Vec<&str> = [device.model.trim(), device.location.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect();
    if !caption.is_empty() {
        let line = Text::new()
            .text(caption.join(" · "))
            .variant(TextVariant::Caption)
            .emphasis(Emphasis::Subtle);
        title = title.child(line);
    }
    let mut row = Stack::new().direction(StackDirection::Horizontal).gap(12.0);
    if !device.emoji.is_empty() {
        row = row.child(Icon::new().emoji(device.emoji.clone()).size(64.0));
    }
    row.child(title).into()
}
