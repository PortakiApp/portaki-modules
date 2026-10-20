//! Réglages hôte (spec Règlement §2) : l'éditeur dérivé de la collection, et ce qui est propre
//! au règlement — la marque de statut, la limite de la carte, le thème suggéré.

use portaki_sdk::prelude::*;
use portaki_sdk::resource::{Choice, CollectionEditor};
use portaki_sdk::sdui::primitives::{Button, Field, FieldHint, Form, Page, Text};

use crate::commands::apply_template::APPLY_TEMPLATE;
use crate::guest::row::hours_label;
use crate::{Rule, RuleStatus, Rules};

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::RulesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Scale
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let rules = Rules::load(&ctx)?;
    let acceptance = Field::new()
        .name("acceptance")
        .label("i18n:host.acceptance.label")
        .child(
            Text::new()
                .text("i18n:host.acceptance.hint")
                .variant(TextVariant::Body),
        );
    let template = Button::new()
        .label("i18n:host.rules.template")
        .variant(ButtonVariant::Outline)
        .action(Action::command(
            &ModuleId::from_static("rules"),
            APPLY_TEMPLATE,
            EmptyArgs {},
        ));
    let editor = CollectionEditor::new(&ctx, &rules)
        .settings("host.display.title", vec![acceptance.into()])
        .hint("host.rules.hint")
        .pill("glyph", "host.column.status")
        .column_with("title", "detail")
        .column("theme")
        .mono("hours")
        .cell("glyph", |r: &Rule| glyph(r.status).into())
        .cell("hours", hours_label)
        .divider_after(rules.meta().limit(), "host.rules.divider")
        .when_empty(template)
        .panel_extra(
            |_, r: &Rule| match r.suggested_theme().filter(|t| *t != r.theme) {
                Some(theme) => {
                    let label = t!(&format!("rule.theme.{}", theme.wire())).unwrap_or_default();
                    let hint =
                        t!("host.rule.theme.suggested", theme = label.as_str()).unwrap_or_default();
                    vec![FieldHint::new().text(hint).into()]
                }
                None => Vec::new(),
            },
        );
    Ok(Surface::new(
        Page::new().child(Form::new().children(editor.build())),
    ))
}

fn glyph(status: RuleStatus) -> &'static str {
    match status {
        RuleStatus::Forbidden => "×",
        RuleStatus::Important => "!",
        RuleStatus::Allowed => "✓",
        RuleStatus::Neutral => "i",
    }
}
