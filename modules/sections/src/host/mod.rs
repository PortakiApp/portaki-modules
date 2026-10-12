//! Host dashboard surface — the « Message d'accueil » card of the listing (spec Bienvenue §2).
//!
//! Placed on `PropertyListingCard`: the shell draws the card frame and its title (`catalog.host
//! .main`) in « Fiche & coordonnées », and the listing's own « Publier » publishes the draft. The
//! module fills it with two `Section`s in the spec's order — the word of welcome, the sections. A
//! rule broken shows under its field (`Field::error`), the same one `publishReadiness` blocks on.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{ButtonVariant, Tone};
use portaki_sdk::sdui::primitives::{
    Avatar, Button, Field, FieldHint, Form, ImageUpload, InlineNotice, KeyValue, Link, Page,
    RadioGroup, RichTextEditor, Section, Stack, StepList, Tag, Text, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, SectionRow, ShowFrom, MAX_SECTIONS};
use crate::content::editor_value;

/// The host's profile page in the dashboard, where the name and the photo are edited.
const PROFILE_HREF: &str = "/settings?tab=profil";

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyListingCard,
    label_key = "catalog.host.main",
    icon = IconName::Home
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let mut sections = config.sections();
    let count = rows_count(&ctx, sections.len());
    sections.resize_with(count, SectionRow::default);
    if ctx.input_bool("template", false) {
        if let Some(first) = sections.first_mut().filter(|row| row.is_blank()) {
            *first = template(&ctx);
        }
    }

    let form = Form::new()
        .child(welcome_card(&config, &ctx))
        .child(sections_card(&config, &sections, &ctx));
    Ok(Surface::new(Page::new().child(form)).with_id(MAIN))
}

/// Rows drawn: the saved ones, or as many as the host asked for, ten at most. Zero is allowed:
/// an empty list offers « Partir d'un modèle » rather than a blank row.
fn rows_count(ctx: &HostContext, saved: usize) -> usize {
    ctx.input_u64("sections_count")
        .map_or(saved, |n| n as usize)
        .min(MAX_SECTIONS)
}

#[derive(Serialize)]
struct SectionsInput {
    sections_count: usize,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    template: bool,
}

fn emit_rows(sections_count: usize, template: bool) -> Action {
    Action::emit(
        contracts::shell::SURFACE_INPUT,
        Some(json_value(SectionsInput {
            sections_count,
            template,
        })),
    )
}

/// §2.1 Mot d'accueil.
fn welcome_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let show_from = ctx
        .input_str("show_from")
        .and_then(ShowFrom::parse)
        .unwrap_or(config.show_from);
    let mut show_field = Field::new()
        .name("show_from")
        .label("i18n:host.showFrom.label")
        .required(true)
        .child(
            RadioGroup::new()
                .name("show_from")
                .value(show_from.as_wire())
                .options(
                    ShowFrom::ALL
                        .into_iter()
                        .map(|option| {
                            ChoiceOption::new(
                                option.as_wire(),
                                format!("i18n:host.showFrom.{}", option.i18n_key()),
                            )
                        })
                        .collect(),
                ),
        );
    if let Some(error) = config.error_of("config.show_from", &ctx.locale) {
        show_field = show_field.error(error);
    }
    Section::new()
        .title("i18n:host.section.welcome")
        .subtitle("i18n:host.section.welcome.help")
        .children(vec![show_field.into(), signature_field(ctx)])
        .into()
}

/// La signature, en lecture seule : le prénom et la photo du profil de l'hôte. La plateforme ne
/// sert ce profil (`ctx.host`) qu'au livret aujourd'hui : sans lui, seul le lien vers le profil.
fn signature_field(ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = Vec::new();
    if let Some(host) = ctx.host.as_ref().filter(|h| !h.name.trim().is_empty()) {
        let mut avatar = Avatar::new().size(40.0);
        if let Some(url) = host.photo_url.as_deref().filter(|u| !u.trim().is_empty()) {
            avatar = avatar.url(url);
        }
        if let Some(initials) = host.initials.as_deref().filter(|i| !i.trim().is_empty()) {
            avatar = avatar.initials(initials);
        }
        children.push(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(10.0)
                .children(vec![
                    avatar.into(),
                    KeyValue::new()
                        .key("i18n:host.signature.name")
                        .value(host.name.trim())
                        .into(),
                ])
                .into(),
        );
    }
    children.push(
        Link::new()
            .label("i18n:host.signature.edit")
            .href(PROFILE_HREF)
            .into(),
    );
    Field::new()
        .label("i18n:host.signature.label")
        .children(children)
        .into()
}

/// §2.2 Sections.
fn sections_card(config: &ModuleConfig, sections: &[SectionRow], ctx: &HostContext) -> Component {
    let rows: Vec<Component> = sections
        .iter()
        .enumerate()
        .map(|(index, row)| section_row(config, index, row, ctx))
        .collect();
    let mut list = StepList::new()
        .label("i18n:host.sections.label")
        .hint(counter(sections.len()))
        .removeLabel("i18n:host.sections.remove")
        .itemKeyPrefix("sections")
        .children(rows);
    if sections.len() < MAX_SECTIONS {
        list = list
            .addLabel("i18n:host.sections.add")
            .addAction(emit_rows(sections.len() + 1, false));
    }
    let mut children: Vec<Component> = Vec::new();
    if sections.is_empty() {
        // Rien d'écrit : la ligne « Le mot de {hôte} » n'apparaît pas dans le livret (§3).
        children.push(
            Text::new()
                .text("i18n:host.sections.empty")
                .variant(TextVariant::Caption)
                .into(),
        );
        children.push(
            Button::new()
                .label("i18n:host.sections.template")
                .variant(ButtonVariant::Outline)
                .action(emit_rows(1, true))
                .into(),
        );
    }
    children.push(list.into());
    if sections.len() >= MAX_SECTIONS {
        children.push(
            Text::new()
                .text("i18n:config.sections.max")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    if let Some(error) = config.error_of("config.sections", &ctx.locale) {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }
    Section::new()
        .title("i18n:host.section.sections")
        .subtitle("i18n:host.sections.help")
        .children(children)
        .into()
}

/// « 2 / 10 » : how many sections of the ten allowed.
fn counter(count: usize) -> String {
    let (count, max) = (count.to_string(), MAX_SECTIONS.to_string());
    match t!("host.sections.count", count = &count, max = &max) {
        Ok(text) if text.contains(&count) => text,
        _ => format!("{count} / {max}"),
    }
}

fn section_row(
    config: &ModuleConfig,
    index: usize,
    row: &SectionRow,
    ctx: &HostContext,
) -> Component {
    let name = |key: &str| format!("sections.{index}.{key}");
    let field = |key: &str, label: &str, child: Component| {
        let mut field = Field::new().name(name(key)).label(label).child(child);
        if let Some(error) =
            config.error_of(&format!("config.sections[{index}].{key}"), &ctx.locale)
        {
            field = field.error(error);
        }
        field
    };
    let id = if row.id.trim().is_empty() {
        format!("section-{}", index + 1)
    } else {
        row.id.clone()
    };
    let saved_title = row.title.host_value(ctx).to_string();
    // « Bienvenue » d'office pour la première, que l'hôte n'a jamais eu à nommer.
    let title = if saved_title.trim().is_empty() && index == 0 {
        t!("host.title.default")
            .ok()
            .filter(|text| text != "host.title.default")
            .unwrap_or_default()
    } else {
        saved_title
    };

    let mut children: Vec<Component> = Vec::new();
    // L'id garde la ligne d'un enregistrement à l'autre : sans lui, la plateforme rapproche par
    // position, et réordonner donnerait une traduction au voisin.
    children.push(TextInput::new().name(name("id")).value(id).into());
    if index == 0 {
        children.push(Tag::new().label("i18n:host.sections.first").into());
    }
    children.push(
        field(
            "title",
            "i18n:host.title.label",
            TextInput::new()
                .name(name("title"))
                .value(title)
                .placeholder("i18n:host.title.placeholder")
                .into(),
        )
        .required(true)
        .into(),
    );
    let mut body: Vec<Component> = vec![
        RichTextEditor::new()
            .name(name("body"))
            .value(editor_value(row.body.host_value(ctx)))
            .into(),
        // L'éditeur riche n'a pas d'indication de saisie : l'exemple de la spec vient dessous.
        FieldHint::new().text("i18n:host.body.placeholder").into(),
    ];
    if index == 0 && config.teaser_too_long() {
        body.push(FieldHint::new().text("i18n:config.body.teaser").into());
    }
    children.push(
        field(
            "body",
            "i18n:host.body.label",
            Stack::new().children(body).into(),
        )
        .required(true)
        .into(),
    );
    children.push(
        field(
            "photo",
            "i18n:host.photo.label",
            ImageUpload::new()
                .name(name("photo"))
                .value(row.photo.clone())
                .into(),
        )
        .into(),
    );
    Stack::new()
        .id(format!("section-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

/// « Partir d'un modèle » (§4) : un mot d'accueil au prénom de l'hôte et au nom du logement.
fn template(ctx: &HostContext) -> SectionRow {
    let property = ctx.property.name.trim();
    let host = ctx
        .host
        .as_ref()
        .map(|h| h.name.trim())
        .filter(|n| !n.is_empty());
    let (key, body) = match host {
        Some(host) => (
            "host.template.body",
            t!("host.template.body", host = host, property = property),
        ),
        None => (
            "host.template.bodyNoHost",
            t!("host.template.bodyNoHost", property = property),
        ),
    };
    let body = body.ok().filter(|text| text != key).unwrap_or_default();
    let title = t!("host.title.default")
        .ok()
        .filter(|text| text != "host.title.default")
        .unwrap_or_default();
    SectionRow {
        id: "section-1".into(),
        title: title.as_str().into(),
        body: editor_value(&body).as_str().into(),
        photo: String::new(),
    }
}
