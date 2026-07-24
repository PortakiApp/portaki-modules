//! Guest home booklet card — consumable shortage form and stay report list.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{
    Button, Card, ChoiceList, Field, Form, ListItem, Stack, Text, TextArea,
};
use portaki_sdk::sdui::surface::Surface;

use super::load::GuestConsumablesData;
use crate::entities::ConsumableReport;
use crate::labels;
use crate::level;

pub fn build_home_card(data: &GuestConsumablesData) -> Surface {
    let mut children: Vec<Component> = Vec::new();

    if data.reports.is_empty() {
        children.push(
            Text::new()
                .text("i18n:home.card.intro")
                .variant(TextVariant::Body)
                .into(),
        );
    } else {
        children.push(
            Text::new()
                .text("i18n:home.card.thanks")
                .variant(TextVariant::Body)
                .into(),
        );
        children.push(
            Text::new()
                .text("i18n:home.card.yourReports")
                .variant(TextVariant::Caption)
                .into(),
        );
        for report in &data.reports {
            children.push(report_list_item(report).into());
        }
    }

    children.push(build_form(data).into());

    Surface::new(
        Card::new()
            .icon("package")
            .title("i18n:home.card.title")
            .child(Stack::new().gap(12.0).children(children)),
    )
    .with_id(crate::ids::HOME_CARD)
}

fn report_list_item(report: &ConsumableReport) -> ListItem {
    let subtitle = level::level_label_key(report.level.as_str());
    ListItem::new()
        .title(report.item_label.clone())
        .subtitle(format!("i18n:{subtitle}"))
}

fn build_form(data: &GuestConsumablesData) -> Form {
    let submit_action = crate::ids::module_id().command_empty(crate::ids::SUBMIT);
    let first_id = data
        .items
        .first()
        .map(|item| item.id.to_string())
        .unwrap_or_default();

    Form::new()
        .child(
            Field::new()
                .name("itemId")
                .label("i18n:form.item.label")
                .required(true)
                .child(item_choice_list(data, &first_id)),
        )
        .child(
            Field::new()
                .name("level")
                .label("i18n:form.level.label")
                .required(true)
                .child(level_choice_list()),
        )
        .child(
            Field::new()
                .name("note")
                .label("i18n:form.note.label")
                .child(
                    TextArea::new()
                        .name("note")
                        .placeholder("i18n:form.note.placeholder"),
                ),
        )
        .child(
            Button::new()
                .label("i18n:form.submit")
                .action(submit_action),
        )
}

fn item_choice_list(data: &GuestConsumablesData, selected: &str) -> ChoiceList {
    let choices: Vec<ChoiceOption> = data
        .items
        .iter()
        .map(|item| {
            let label = labels::pick_label(
                &labels::labels_from_item(item),
                &data.locale,
                &data.property_locale,
            );
            ChoiceOption::new(item.id.to_string(), label).icon("package")
        })
        .collect();

    let mut list = ChoiceList::new()
        .name("itemId")
        .layout(ChoiceListLayout::Compact)
        .choices(choices);
    if !selected.is_empty() {
        list = list.value(selected);
    }
    list
}

fn level_choice_list() -> ChoiceList {
    ChoiceList::new()
        .name("level")
        .layout(ChoiceListLayout::Compact)
        .value(level::DEFAULT)
        .choices(vec![
            ChoiceOption::new("missing", "i18n:form.level.missing").icon("circle-x"),
            ChoiceOption::new("low", "i18n:form.level.low").icon("gauge"),
        ])
}
