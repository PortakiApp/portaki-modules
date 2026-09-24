//! Module commands — host list editor, guest complete / uncomplete.

use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::EditableListItem;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::labels::Labels;
use crate::lists;
use crate::storage;

/// Workspace header Save → the list selected in the editor.
///
/// `items` is the `EditableList` value: a JSON string (or array) of
/// `[{ id?, label, labelEn?, photo? }]`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateConfigArgs {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub placement: String,
    /// « Julie Martin · Ménage » — name, then role after the middle dot.
    #[serde(default)]
    pub assignee: String,
    #[serde(default)]
    pub deadline: String,
    #[serde(default)]
    pub notify_assignee: Option<Value>,
    #[serde(default)]
    pub alert_host: Option<Value>,
    #[serde(default)]
    pub items: Option<Value>,
}

/// Saves the selected list. Without a known `id` there is nothing to save (the « new » panel
/// creates lists with `createChecklist`).
#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let Some(mut list) = Uuid::parse_str(args.id.trim()).ok().and_then(|id| {
        storage::list_checklists()
            .ok()?
            .into_iter()
            .find(|l| l.id == id)
    }) else {
        return Ok(());
    };
    let host = list.audience == lists::HOST;
    let name = args.name.trim();
    // A renamed list is named in one language; an untouched one keeps its translation.
    if !name.is_empty() && name != list.name_fr && name != list.name_en {
        list.name_fr = name.to_string();
        list.name_en = name.to_string();
    }
    if host {
        list.trigger = lists::pick(&args.trigger, lists::HOST_TRIGGERS).to_string();
        list.deadline = Some(lists::pick(&args.deadline, lists::DEADLINES).to_string());
        let (name, role) = split_assignee(&args.assignee);
        list.assignee_name = name;
        list.assignee_role = role;
        list.notify_assignee = flag(args.notify_assignee.as_ref(), list.notify_assignee);
        list.alert_host = flag(args.alert_host.as_ref(), list.alert_host);
    } else {
        list.trigger = lists::pick(&args.trigger, lists::GUEST_TRIGGERS).to_string();
        list.placement = lists::pick(&args.placement, lists::PLACEMENTS).to_string();
    }
    let items = parse_items(args.items.as_ref())?;
    storage::save_checklist(list.clone())?;
    let rows = items
        .into_iter()
        .filter(|item| !item.label.trim().is_empty())
        .map(|item| {
            let fr = item.label.trim().to_string();
            // A host list is written in one language: the same text serves both.
            let en = item
                .label_en
                .map(|en| en.trim().to_string())
                .filter(|en| !en.is_empty() && !host)
                .unwrap_or_else(|| fr.clone());
            let id = item.id.and_then(|id| Uuid::parse_str(&id).ok());
            let labels = Labels::from([("fr".to_string(), fr), ("en".to_string(), en)]);
            (id, labels, host && item.photo == Some(true))
        })
        .collect();
    storage::replace_items(list.id, rows)
}

fn parse_items(raw: Option<&Value>) -> Result<Vec<EditableListItem>> {
    let invalid = |error: serde_json::Error| PortakiError::Host(format!("invalid_items: {error}"));
    match raw {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::String(json)) if json.trim().is_empty() => Ok(Vec::new()),
        Some(Value::String(json)) => serde_json::from_str(json).map_err(invalid),
        Some(value) => serde_json::from_value(value.clone()).map_err(invalid),
    }
}

/// Form toggles arrive as booleans, sometimes as `"true"` / `"false"`.
fn flag(raw: Option<&Value>, fallback: bool) -> bool {
    match raw {
        Some(Value::Bool(value)) => *value,
        Some(Value::String(value)) => value == "true",
        _ => fallback,
    }
}

fn split_assignee(raw: &str) -> (Option<String>, Option<String>) {
    let clean = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
    match raw.split_once('·') {
        Some((name, role)) => (clean(name), clean(role)),
        None => (clean(raw), None),
    }
}

/// Arguments for `createChecklist`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct CreateChecklistArgs {
    /// One of [`lists::TEMPLATES`].
    pub template: String,
}

#[portaki_sdk::command(name = "createChecklist")]
pub fn create_checklist(_ctx: Context, args: CreateChecklistArgs) -> Result<()> {
    let template = lists::template(&args.template)
        .ok_or_else(|| PortakiError::Host(format!("unknown_template:{}", args.template)))?;
    storage::create_from_template(template).map(|_| ())
}

/// Arguments for `deleteChecklist`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct DeleteChecklistArgs {
    pub id: Uuid,
}

#[portaki_sdk::command(name = "deleteChecklist")]
pub fn delete_checklist(_ctx: Context, args: DeleteChecklistArgs) -> Result<()> {
    storage::delete_checklist(args.id)
}

/// Arguments for complete / uncomplete.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct ItemIdArgs {
    pub item_id: Uuid,
}

#[portaki_sdk::command(name = "completeItem", guest)]
pub fn complete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    storage::complete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}

#[portaki_sdk::command(name = "uncompleteItem", guest)]
pub fn uncomplete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    storage::uncomplete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}

fn require_stay_id(ctx: &Context) -> Result<Uuid> {
    ctx.guest
        .as_ref()
        .map(|guest| guest.session_id)
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))
}

#[portaki_sdk::wire(serialize)]
struct ProgressPayload {
    property_id: Uuid,
    percentage: u8,
}

#[portaki_sdk::wire(serialize)]
struct CompletedPayload {
    stay_id: Uuid,
}

/// Progress of the stay over every guest item.
fn emit_progress(property_id: Uuid, stay_id: Uuid) -> Result<()> {
    let guest_items = crate::queries::guest_items()?;
    let completed = storage::list_completions(Some(stay_id))?;
    let done = guest_items
        .iter()
        .filter(|item| completed.iter().any(|row| row.item_id == item.id))
        .count();
    let percentage = ((done * 100).checked_div(guest_items.len()).unwrap_or(0)) as u8;
    events::emit(
        crate::ids::PROGRESS_UPDATED,
        &ProgressPayload {
            property_id,
            percentage,
        },
    )?;
    if percentage == 100 {
        events::emit(crate::ids::COMPLETED, &CompletedPayload { stay_id })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignee_splits_on_the_middle_dot() {
        assert_eq!(
            split_assignee("Julie Martin · Ménage"),
            (Some("Julie Martin".into()), Some("Ménage".into()))
        );
        assert_eq!(split_assignee("  "), (None, None));
        assert_eq!(split_assignee("Paul"), (Some("Paul".into()), None));
    }

    #[test]
    fn items_parse_from_a_json_string_or_an_array() {
        let json = Value::String(r#"[{"label":"Clés","labelEn":"Keys","photo":true}]"#.into());
        let items = parse_items(Some(&json)).unwrap();
        assert_eq!(items[0].label_en.as_deref(), Some("Keys"));
        assert_eq!(items[0].photo, Some(true));
        let array = serde_json::json!([{ "id": "x", "label": "Sols" }]);
        assert_eq!(parse_items(Some(&array)).unwrap()[0].label, "Sols");
        assert!(flag(Some(&Value::String("true".into())), false));
    }
}
