//! Host dashboard surface — design `sections-editor-v1` (master-detail SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{ButtonVariant, Tone};
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Field, FieldHint, Form, List, ListItem, Page, RichTextEditor, Stack,
    Text, TextInput,
};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{body_block_count, body_plain_text, editor_value};
use crate::model::{lang_code, SectionView};
use crate::store;

const SELECT_NEW: &str = "__new__";

/// Leading icons cycle — matches design list (`home`, `map-pin`, `star`).
const LIST_ICONS: [&str; 3] = ["home", "map-pin", "star"];

/// Host editor — sections list (left) + selected section form (right).
#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = lang_code(&ctx.locale);
    let property_locale = ctx.property.locale.clone();
    let sections = store::list_all(&ctx.locale, &property_locale).unwrap_or_default();

    let mut selected_id = ctx.input_str("selectedId").unwrap_or("").to_string();
    if selected_id.is_empty() {
        if let Some(first) = sections.first() {
            selected_id = first.id.to_string();
        }
    }

    let list_card = build_list_card(&sections, &selected_id, &lang);
    let detail_panel = build_detail_panel(&sections, &selected_id, &lang);

    Surface::new(
        Page::new().child(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(16.0)
                .children(vec![list_card, detail_panel]),
        ),
    )
    .with_id(crate::ids::HOST_MAIN)
}

#[portaki_sdk::wire(serialize)]
struct SurfaceInputSelectedId<'a> {
    selected_id: &'a str,
}

fn emit_select(selected_id: &str) -> Action {
    Action::emit(
        contracts::shell::SURFACE_INPUT,
        Some(json_value(SurfaceInputSelectedId { selected_id })),
    )
}

fn build_list_card(sections: &[SectionView], selected_id: &str, lang: &str) -> Component {
    let mut stack_children: Vec<Component> = Vec::new();

    if sections.is_empty() {
        stack_children.push(Component::Text(
            Text::new()
                .text("i18n:host.list.empty")
                .variant(TextVariant::Caption),
        ));
    } else {
        let items: Vec<Component> = sections
            .iter()
            .enumerate()
            .map(|(index, section)| {
                let id = section.id.to_string();
                let title = section_title(section);
                let subtitle = list_subtitle(index, section, lang);
                let mut item = ListItem::new()
                    .title(title)
                    .subtitle(subtitle)
                    .leading(LIST_ICONS[index % LIST_ICONS.len()])
                    .chevron(true)
                    .action(emit_select(&id));
                if selected_id == id {
                    item = item.tone(Tone::Primary);
                }
                Component::ListItem(item)
            })
            .collect();
        stack_children.push(Component::List(List::new().children(items)));
    }

    let mut add = Button::new()
        .label("i18n:host.list.add")
        .variant(ButtonVariant::Ghost)
        .action(emit_select(SELECT_NEW));
    if selected_id == SELECT_NEW {
        add = add.tone(Tone::Primary);
    }
    stack_children.push(Component::Button(add));

    Component::Card(
        Card::new()
            .title("i18n:host.list.title")
            .child(Stack::new().gap(10.0).children(stack_children)),
    )
}

fn build_detail_panel(sections: &[SectionView], selected_id: &str, lang: &str) -> Component {
    if selected_id.is_empty() {
        return Component::Card(
            Card::new().title("i18n:host.detail.card.title").child(
                EmptyState::new()
                    .title("i18n:host.detail.empty.title")
                    .description("i18n:host.detail.empty.description")
                    .icon("home"),
            ),
        );
    }

    let is_new = selected_id == SELECT_NEW;
    let section = if is_new {
        None
    } else {
        sections.iter().find(|s| s.id.to_string() == selected_id)
    };

    if !is_new && section.is_none() {
        return Component::Card(
            Card::new().title("i18n:host.detail.card.title").child(
                EmptyState::new()
                    .title("i18n:host.detail.missing.title")
                    .description("i18n:host.detail.missing.description")
                    .icon("home"),
            ),
        );
    }

    let id = section.map(|s| s.id.to_string()).unwrap_or_default();
    let (title, body) = section
        .map(|s| locale_fields(s, lang))
        .unwrap_or_else(|| default_new_fields(lang));
    let body_editor = editor_value(&body);

    let save_action = crate::ids::module_id().command_empty(crate::ids::SAVE_SECTION);
    let cancel_target = if is_new {
        sections
            .first()
            .map(|s| s.id.to_string())
            .unwrap_or_default()
    } else {
        id.clone()
    };
    let cancel_action = emit_select(&cancel_target);

    let card_title = if is_new {
        "i18n:host.detail.new.title"
    } else if section.map(|s| s.sort_order).unwrap_or(1) == 0 {
        "i18n:host.detail.primary.title"
    } else {
        "i18n:host.detail.edit.title"
    };

    let form_children: Vec<Component> = vec![
        TextInput::new().name("id").value(id).into(),
        TextInput::new().name("lang").value(lang).into(),
        Field::new()
            .name("title")
            .label("i18n:host.title.label")
            .child(TextInput::new().name("title").value(title))
            .into(),
        Field::new()
            .name("body_markdown")
            .label("i18n:host.body.label")
            .child(
                RichTextEditor::new()
                    .name("body_markdown")
                    .value(body_editor),
            )
            .into(),
        FieldHint::new().text("i18n:host.body.hint").into(),
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(10.0)
            .children(vec![
                Button::new()
                    .label("i18n:host.cancel")
                    .variant(ButtonVariant::Outline)
                    .action(cancel_action)
                    .into(),
                Button::new()
                    .label("i18n:host.save")
                    .tone(Tone::Primary)
                    .action(save_action)
                    .into(),
            ])
            .into(),
    ];

    Component::Card(
        Card::new()
            .title(card_title)
            .subtitle("i18n:host.detail.bar")
            .child(Form::new().children(form_children)),
    )
}

fn section_title(section: &SectionView) -> String {
    let title = section.title.trim();
    if title.is_empty() {
        "i18n:host.list.untitled".to_string()
    } else {
        section.title.clone()
    }
}

fn list_subtitle(index: usize, section: &SectionView, lang: &str) -> String {
    if index == 0 || section.sort_order == 0 {
        return "i18n:host.list.primary".to_string();
    }
    let blocks = body_block_count(&section.body_markdown);
    if lang == "en" {
        match blocks {
            0 => "Empty".into(),
            1 => "1 paragraph".into(),
            n => format!("{n} paragraphs"),
        }
    } else {
        match blocks {
            0 => "Vide".into(),
            1 => "1 paragraphe".into(),
            n => format!("{n} paragraphes"),
        }
    }
}

fn locale_fields(section: &SectionView, lang: &str) -> (String, String) {
    let row = section.locales.iter().find(|l| lang_code(&l.lang) == lang);
    let title = row
        .map(|l| l.title.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| section.title.clone());
    let body = row
        .map(|l| l.body_markdown.clone())
        .filter(|t| !body_plain_text(t).trim().is_empty())
        .unwrap_or_else(|| section.body_markdown.clone());
    (title, body)
}

fn default_new_fields(lang: &str) -> (String, String) {
    if lang == "en" {
        (
            "Welcome".into(),
            "Welcome to L'Islette — the whole team wishes you a great stay.".into(),
        )
    } else {
        (
            "Bienvenue".into(),
            "Bienvenue à L'Islette — toute l'équipe vous souhaite un excellent séjour.".into(),
        )
    }
}
