//! Module commands — host list editor, guest complete / uncomplete.

use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::labels::{self, Labels};
use crate::lists;
use crate::storage;

/// Une ligne du formulaire hôte, telle que la primitive `EditableList` la sérialise.
///
/// Le module lit sa propre forme plutôt que `EditableListItem` du SDK : les deux champs du §2.9
/// arrivent par le fil dès que le dashboard les envoie, sans attendre une version du SDK.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostRow {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    label: String,
    #[serde(default)]
    label_en: Option<String>,
    #[serde(default)]
    photo: Option<bool>,
    /// Rubrique de l'étape (§2.9), dans la langue éditée.
    #[serde(default)]
    group: Option<String>,
    /// Précision sous le libellé (§2.9), dans la langue éditée.
    #[serde(default)]
    description: Option<String>,
}

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
    /// Guest lists only: « Étapes visibles » (3 à 10), a number or its text.
    #[serde(default)]
    pub visible_limit: Option<Value>,
    /// Guest lists only: « Message final », in the language edited.
    #[serde(default)]
    pub done_message: Option<String>,
    /// Guest lists only: « Rappel le matin du départ ».
    #[serde(default)]
    pub remind: Option<Value>,
}

/// Saves the selected list. Without a known `id` there is nothing to save (the « new » panel
/// creates lists with `createChecklist`).
#[portaki_sdk::command(
    name = "updateConfig",
    example(
        label = "Renommer la liste de départ",
        input = r#"{"id":"3f2b8c1e-7a4d-4e9b-9c61-2d5e8f0a1b47","name":"Avant de partir","items":[{"label":"Fermer les fenêtres"},{"label":"Sortir les poubelles"}]}"#
    )
)]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
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
    let lang = labels::lang_code(&ctx.locale);
    if !host {
        let mut display = storage::display::read(list.id);
        if let Some(limit) = number(args.visible_limit.as_ref()) {
            // Kept as typed, out of bounds too: the editor says « Entre 3 et 10. » under it.
            display.visible_limit = Some(limit.round().clamp(0.0, f64::from(u32::MAX)) as u32);
        }
        if args.done_message.is_some() {
            display.done_message = labels::encode_map(&merge_lang(
                &lang,
                args.done_message,
                Some(&display.done_message),
            ));
        }
        if args.remind.is_some() {
            display.remind = Some(flag(args.remind.as_ref(), display.remind()));
        }
        storage::display::write(list.id, &display)?;
    }
    let items = parse_items(args.items.as_ref())?;
    let existing_items = storage::all_items_of(list.id)?;
    storage::save_checklist(list.clone())?;
    // A blank step is kept: `publishReadiness` asks for it (« Écrivez l'étape. »), and the readers
    // skip it until it is written (`storage::list_items`).
    let rows = items
        .into_iter()
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
            // Le formulaire n'édite qu'une langue à la fois pour ces deux champs : on écrit celle
            // de la requête et on garde ce que les autres langues contenaient déjà, sinon passer
            // en anglais pour corriger une faute effacerait le groupe français.
            let previous = id.and_then(|id| existing_items.iter().find(|row| row.id == id));
            storage::ItemDraft {
                id,
                labels,
                group: merge_lang(
                    &lang,
                    item.group,
                    previous.map(|row| row.group_i18n.as_str()),
                ),
                description: merge_lang(
                    &lang,
                    item.description,
                    previous.map(|row| row.description_i18n.as_str()),
                ),
                photo_required: host && item.photo == Some(true),
            }
        })
        .collect();
    storage::replace_items(list.id, rows)
}

/// Writes the text typed in `lang` over the stored language map and keeps the other languages:
/// the form edits one language at a time, and switching to English to fix a typo must not wipe
/// the French. `None` leaves the map untouched, a blank text removes `lang`.
fn merge_lang(lang: &str, typed: Option<String>, stored: Option<&str>) -> Labels {
    let mut map = stored.map(labels::decode_map).unwrap_or_default();
    match typed.map(|value| value.trim().to_string()) {
        Some(value) if value.is_empty() => {
            map.remove(lang);
        }
        Some(value) => {
            map.insert(lang.to_string(), value);
        }
        None => {}
    }
    map
}

/// A number field arrives as a number, sometimes as its text; blank = untouched.
fn number(raw: Option<&Value>) -> Option<f64> {
    match raw? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
    .filter(|n: &f64| n.is_finite())
}

fn parse_items(raw: Option<&Value>) -> Result<Vec<HostRow>> {
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

#[portaki_sdk::command(
    name = "createChecklist",
    example(label = "Liste de départ", input = r#"{"template":"departure"}"#),
    example(
        label = "Ménage entre deux séjours",
        input = r#"{"template":"cleaning"}"#
    )
)]
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

#[portaki_sdk::command(
    name = "deleteChecklist",
    example(
        label = "Supprimer une liste",
        input = r#"{"id":"3f2b8c1e-7a4d-4e9b-9c61-2d5e8f0a1b47"}"#
    )
)]
pub fn delete_checklist(_ctx: Context, args: DeleteChecklistArgs) -> Result<()> {
    storage::delete_checklist(args.id)
}

/// Arguments for complete / uncomplete.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct ItemIdArgs {
    pub item_id: Uuid,
}

#[portaki_sdk::command(
    name = "completeItem",
    guest,
    example(
        label = "Cocher une tâche",
        input = r#"{"itemId":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c"}"#
    )
)]
pub fn complete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    require_guest_item(args.item_id)?;
    storage::complete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}

#[portaki_sdk::command(
    name = "uncompleteItem",
    guest,
    example(
        label = "Décocher une tâche",
        input = r#"{"itemId":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c"}"#
    )
)]
pub fn uncomplete_item(ctx: Context, args: ItemIdArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    require_guest_item(args.item_id)?;
    storage::uncomplete_item(stay_id, args.item_id)?;
    emit_progress(ctx.property_id, stay_id)
}

/// Les étapes cochées, telles que la primitive `ChoiceList` les sérialise : une liste de valeurs
/// séparées par des virgules.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct SetCompletedArgs {
    #[serde(default)]
    pub item_ids: String,
}

/// Remplace l'ensemble des étapes cochées d'un séjour.
///
/// Un `ChoiceList` multiple est un ensemble, pas un événement : il renvoie tout ce qui est coché à
/// chaque basculement. Remplacer est donc idempotent — rejouer le même appel ne double rien.
///
/// ponytail: dernier écrit gagne. Deux téléphones qui cochent chacun une étape au même instant
/// s'écrasent l'un l'autre ; pour une liste de départ remplie par un voyageur sur son téléphone,
/// l'échange ne vaut pas un journal d'opérations.
#[portaki_sdk::command(
    name = "setCompleted",
    guest,
    example(
        label = "Deux étapes cochées",
        input = r#"{"itemIds":"8c0e5a4b-1d2f-4e6a-9b7c-3f1d2e4a5b6c,1f8e7d6c-5b4a-4938-8271-6e5d4c3b2a19"}"#
    ),
    example(label = "Tout décoché", input = r#"{"itemIds":""}"#)
)]
pub fn set_completed(ctx: Context, args: SetCompletedArgs) -> Result<()> {
    let stay_id = require_stay_id(&ctx)?;
    let guest_items = crate::queries::guest_items()?;
    let mut wanted = Vec::new();
    for raw in args.item_ids.split(',') {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let id = Uuid::parse_str(trimmed)
            .map_err(|_| PortakiError::Host("invalid_item_id".to_string()))?;
        if !guest_items.iter().any(|item| item.id == id) {
            return Err(PortakiError::Host("not_guest_item".to_string()));
        }
        if !wanted.contains(&id) {
            wanted.push(id);
        }
    }

    let already: Vec<Uuid> = storage::list_completions(Some(stay_id))?
        .into_iter()
        .map(|row| row.item_id)
        .collect();
    for id in &already {
        if !wanted.contains(id) && guest_items.iter().any(|item| item.id == *id) {
            storage::uncomplete_item(stay_id, *id)?;
        }
    }
    for id in &wanted {
        if !already.contains(id) {
            storage::complete_item(stay_id, *id)?;
        }
    }
    emit_progress(ctx.property_id, stay_id)
}

/// A guest ticks only the lists written for guests, never the host's.
fn require_guest_item(item_id: Uuid) -> Result<()> {
    if crate::queries::guest_items()?
        .iter()
        .any(|item| item.id == item_id)
    {
        Ok(())
    } else {
        Err(PortakiError::Host("not_guest_item".to_string()))
    }
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
