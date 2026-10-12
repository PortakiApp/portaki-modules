//! Module commands — save / delete / reorder appliances.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::{Appliance, ApplianceStatus, ApplianceStep, MAX_APPLIANCES};
use crate::store;

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveApplianceArgs {
    pub id: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub emoji: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub featured: bool,
    pub order: Option<i32>,
    #[serde(default)]
    pub location: String,
    #[serde(default, rename = "manualUrl")]
    pub manual_url: String,
    #[serde(default, rename = "safetyNote")]
    pub safety_note: String,
    #[serde(default)]
    pub status: ApplianceStatus,
    /// « Marque et modèle », non traduit.
    #[serde(default)]
    pub model: String,
    #[serde(default, rename = "videoUrl", alias = "video_url")]
    pub video_url: String,
    /// La valeur de l'`EditableList` « Étapes » : `[{ label }]`, en JSON ou en tableau, ou une
    /// liste de textes. Absente : les étapes restent telles quelles.
    #[serde(default)]
    pub steps: Option<serde_json::Value>,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteApplianceArgs {
    pub id: String,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderAppliancesArgs {
    #[serde(rename = "orderedIds")]
    pub ordered_ids: Vec<String>,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSafetyNoticeArgs {
    #[serde(default, rename = "safetyNotice", alias = "safety_notice")]
    pub safety_notice: String,
    #[serde(
        default,
        rename = "paperManualsLocation",
        alias = "paper_manuals_location"
    )]
    pub paper_manuals_location: String,
    /// « Appareils mis en avant » : un nombre, le `NumberInput` n'envoie pas d'entier.
    #[serde(default, rename = "featuredLimit", alias = "featured_limit")]
    pub featured_limit: Option<f64>,
}

/// Host SDUI form payload — nested `devices.N.*` + `safetyNotice`.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaceDevicesArgs {
    /// Absente = laissée telle quelle, comme `paperManualsLocation`.
    #[serde(default, rename = "safetyNotice", alias = "safety_notice")]
    pub safety_notice: Option<String>,
    /// Absent = laissée telle quelle. Un formulaire qui n'envoie que les appareils ne doit pas
    /// effacer l'emplacement des notices papier au passage.
    #[serde(
        default,
        rename = "paperManualsLocation",
        alias = "paper_manuals_location"
    )]
    pub paper_manuals_location: Option<String>,
    #[serde(default)]
    pub devices: Vec<ReplaceDeviceSlot>,
}

/// Une ligne de `replaceDevices`. Un champ absent garde la valeur enregistrée de l'appareil de
/// même `id` ; une valeur vide l'efface.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplaceDeviceSlot {
    #[serde(default)]
    pub id: String,
    /// Absent : gardé (une ligne du tableau jamais ouverte n'envoie que son id). Vide : la ligne
    /// est retirée si elle n'envoie rien d'autre.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Absent ou `null` : gardé.
    #[serde(default)]
    pub featured: serde_json::Value,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default, rename = "manualUrl", alias = "manual_url")]
    pub manual_url: Option<String>,
    #[serde(default, rename = "safetyNote", alias = "safety_note")]
    pub safety_note: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    /// « Actif » (case du tableau, interrupteur du panneau) : prime sur `status`.
    #[serde(default)]
    pub active: serde_json::Value,
    /// « Notice PDF » : la référence `portaki-file:` du `FileUpload`.
    #[serde(default, rename = "manualFile", alias = "manual_file")]
    pub manual_file: Option<String>,
    /// « Marque et modèle », non traduit.
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "videoUrl", alias = "video_url")]
    pub video_url: Option<String>,
    /// La valeur de l'`EditableList` « Étapes » : `[{ label }]`, en JSON ou en tableau, ou une
    /// liste de textes.
    #[serde(default)]
    pub steps: Option<serde_json::Value>,
}

#[portaki_sdk::command(
    name = "saveAppliance",
    example(
        label = "Ajouter le lave-linge",
        input = r#"{"name":"Lave-linge","emoji":"🌀","description":"Programme « Coton 40° » pour un lavage courant.","location":"Salle de bain","safetyNote":"Merci de ne pas lancer de machine après 22 h."}"#
    ),
    example(
        label = "Mettre en avant les plaques",
        input = r#"{"name":"Plaques à induction","emoji":"🍳","featured":true,"location":"Cuisine","manualUrl":"https://example.com/notice-plaques.pdf"}"#
    )
)]
pub fn save_appliance(ctx: Context, args: SaveApplianceArgs) -> Result<Appliance> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let name = args.name.trim().to_string();
    if name.is_empty() {
        return Err(refusal(&ctx, "host.device.name.required", &[]));
    }

    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    let is_create = args
        .id
        .as_ref()
        .map(|id| id.trim().is_empty() || !payload.devices.iter().any(|d| d.id == *id))
        .unwrap_or(true);

    if is_create && payload.devices.len() >= MAX_APPLIANCES {
        return Err(refusal(&ctx, "host.devices.tooMany", &[]));
    }

    let id = args
        .id
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let order = args.order.unwrap_or_else(|| {
        payload
            .devices
            .iter()
            .map(|d| d.order)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0)
    });

    let previous = payload.find_device(&id);
    let next = Appliance {
        id: id.clone(),
        name,
        emoji: args.emoji.trim().to_string(),
        description: normalize_description(&args.description),
        featured: args.featured,
        order,
        location: args.location.trim().to_string(),
        manual_url: args.manual_url.trim().to_string(),
        safety_note: args.safety_note.trim().to_string(),
        status: args.status,
        model: args.model.trim().to_string(),
        video_url: args.video_url.trim().to_string(),
        steps: parse_steps(args.steps.as_ref())
            .unwrap_or_else(|| previous.map(|d| d.steps.clone()).unwrap_or_default()),
        manual_file: previous.map(|d| d.manual_file.clone()).unwrap_or_default(),
    };

    if next.featured {
        let featured_others = payload
            .devices
            .iter()
            .filter(|d| d.id != id && d.featured && d.status != ApplianceStatus::Hidden)
            .count();
        let limit = payload.featured_limit();
        if featured_others >= limit {
            let count = limit.to_string();
            return Err(refusal(&ctx, "host.featured.tooMany", &[("count", &count)]));
        }
    }

    if let Some(existing) = payload.devices.iter_mut().find(|d| d.id == id) {
        *existing = next.clone();
    } else {
        payload.devices.push(next.clone());
    }

    payload.sort_by_order();
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(next)
}

#[portaki_sdk::command(
    name = "deleteAppliance",
    example(label = "Retirer le lave-linge", input = r#"{"id":"lave-linge"}"#)
)]
pub fn delete_appliance(ctx: Context, args: DeleteApplianceArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let id = args.id.trim();
    if id.is_empty() {
        return Err(PortakiError::Host("appliance id is required".into()));
    }
    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    let before = payload.devices.len();
    payload.devices.retain(|d| d.id != id);
    if payload.devices.len() == before {
        return Err(PortakiError::Host(format!("appliance not found: {id}")));
    }
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

#[portaki_sdk::command(
    name = "reorderAppliances",
    example(
        label = "Plaques en premier",
        input = r#"{"orderedIds":["plaques-a-induction","lave-linge"]}"#
    )
)]
pub fn reorder_appliances(ctx: Context, args: ReorderAppliancesArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    if args.ordered_ids.is_empty() {
        return Ok(());
    }
    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    for (index, id) in args.ordered_ids.iter().enumerate() {
        if let Some(device) = payload.devices.iter_mut().find(|d| d.id == *id) {
            device.order = index as i32;
        }
    }
    payload.sort_by_order();
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

#[portaki_sdk::command(
    name = "saveSafetyNotice",
    example(
        label = "Consigne de sécurité",
        input = r#"{"safetyNotice":"Coupez le disjoncteur général avant toute intervention."}"#
    )
)]
pub fn save_safety_notice(ctx: Context, args: SaveSafetyNoticeArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    payload.safety_notice = normalize_description(&args.safety_notice);
    payload.paper_manuals_location = args.paper_manuals_location.trim().to_string();
    // Enregistré tel quel : hors de 2 à 6, c'est `problems()` qui le dit sous le champ et bloque
    // la publication ; la carte, elle, borne à la lecture.
    if let Some(limit) = args.featured_limit.filter(|n| n.is_finite()) {
        payload.featured_limit = Some(limit.round() as u32);
    }
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

/// Replace the full device list from the host SDUI form (empty name = drop slot).
#[portaki_sdk::command(
    name = "replaceDevices",
    example(
        label = "Formulaire complet",
        input = r#"{"safetyNotice":"Coupez le disjoncteur avant toute intervention.","devices":[{"name":"Plaques à induction","emoji":"🍳","featured":true,"location":"Cuisine"},{"name":"Télévision","emoji":"📺","location":"Salon","status":"active"}]}"#
    )
)]
pub fn replace_devices(ctx: Context, args: ReplaceDevicesArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let mut payload = store::load_payload_for(&lang, &ctx.property.locale)?;
    let next_devices = merge_devices(&ctx, &payload, &args.devices)?;

    let featured_count = next_devices
        .iter()
        .filter(|d| d.featured && d.status != ApplianceStatus::Hidden)
        .count();
    let limit = payload.featured_limit();
    if featured_count > limit {
        let count = limit.to_string();
        return Err(refusal(&ctx, "host.featured.tooMany", &[("count", &count)]));
    }

    if let Some(notice) = args.safety_notice.as_deref() {
        payload.safety_notice = normalize_description(notice);
    }
    if let Some(location) = args.paper_manuals_location.as_deref() {
        payload.paper_manuals_location = location.trim().to_string();
    }
    payload.devices = next_devices;
    payload.sort_by_order();
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

/// Le formulaire des réglages (page du module, onglet Appareils) : enregistré à chaque saisie par
/// l'auto-save du tableau de bord.
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateConfigArgs {
    /// « Appareils mis en avant » : un nombre, le `NumberInput` n'envoie pas d'entier.
    #[serde(default, rename = "featuredLimit", alias = "featured_limit")]
    pub featured_limit: Option<f64>,
    /// « Ordre des pièces » : la valeur de l'`EditableList`, `[{ label }]` en JSON ou en tableau.
    #[serde(default, rename = "roomOrder", alias = "room_order")]
    pub room_order: Option<serde_json::Value>,
    #[serde(
        default,
        rename = "paperManualsLocation",
        alias = "paper_manuals_location"
    )]
    pub paper_manuals_location: Option<String>,
    /// Absente : la liste reste telle quelle. Présente : les lignes envoyées, dans cet ordre.
    #[serde(default)]
    pub devices: Option<Vec<ReplaceDeviceSlot>>,
}

/// Enregistre les réglages. Rien n'est refusé ici : ce qui dépasse une borne (trop d'appareils en
/// avant, texte trop long) s'affiche sous son champ et bloque « Publier » (`publishReadiness`) —
/// un auto-save refusé perdrait la saisie en cours.
#[portaki_sdk::command(
    name = "updateConfig",
    example(
        label = "Carte d'accueil et deux appareils",
        input = r#"{"featuredLimit":4,"paperManualsLocation":"Boîte rouge, étagère du salon","devices":[{"id":"four","name":"Four","location":"Cuisine","featured":true,"active":true},{"id":"lave-linge"}]}"#
    )
)]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = crate::content::AppliancesBundle::lang_code(&ctx.locale);
    let mut payload = store::load_host_payload(&lang, &ctx.property.locale)?;
    if let Some(limit) = args.featured_limit.filter(|n| n.is_finite()) {
        payload.featured_limit = Some(limit.round().max(0.0) as u32);
    }
    if let Some(order) = args.room_order.as_ref() {
        payload.room_order = parse_labels(order);
    }
    if let Some(location) = args.paper_manuals_location.as_deref() {
        payload.paper_manuals_location = location.trim().to_string();
    }
    if let Some(devices) = args.devices.as_deref() {
        payload.devices = merge_devices(&ctx, &payload, devices)?;
    }
    let _ = store::save_payload_for(&lang, &payload)?;
    Ok(())
}

/// Les lignes envoyées, fusionnées aux appareils enregistrés de même `id` : un champ absent garde
/// sa valeur, un champ vide l'efface. Une ligne vide (nom vide, rien d'autre) est retirée.
fn merge_devices(
    ctx: &Context,
    payload: &crate::content::AppliancesPayload,
    slots: &[ReplaceDeviceSlot],
) -> Result<Vec<Appliance>> {
    let mut next_devices: Vec<Appliance> = Vec::new();
    for (index, slot) in slots.iter().enumerate() {
        let id = slot.id.trim();
        let stored = (!id.is_empty()).then(|| payload.find_device(id)).flatten();
        let name = match (slot.name.as_deref(), stored) {
            (Some(sent), _) => sent.trim().to_string(),
            (None, Some(d)) => d.name.clone(),
            (None, None) => String::new(),
        };
        if name.is_empty() && (stored.is_none() || slot.name.is_some()) && slot_is_blank(slot) {
            continue;
        }
        if next_devices.len() >= MAX_APPLIANCES {
            return Err(refusal(ctx, "host.devices.tooMany", &[]));
        }
        let id = if id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            id.to_string()
        };

        // Ce que la ligne n'envoie pas reste tel qu'enregistré : un formulaire sans champ
        // « Sécurité » effaçait la consigne de chaque appareil à chaque enregistrement.
        let text = |sent: &Option<String>, kept: fn(&Appliance) -> &String| {
            sent.as_deref()
                .map(|value| value.trim().to_string())
                .or_else(|| stored.map(|d| kept(d).clone()))
                .unwrap_or_default()
        };
        let featured = match &slot.featured {
            serde_json::Value::Null => stored.is_some_and(|d| d.featured),
            value => parse_boolish(value),
        };
        let status = match (&slot.active, slot.status.as_deref()) {
            (serde_json::Value::Null, Some(status))
                if status.trim().eq_ignore_ascii_case("hidden") =>
            {
                ApplianceStatus::Hidden
            }
            (serde_json::Value::Null, Some(_)) => ApplianceStatus::Active,
            (serde_json::Value::Null, None) => stored.map(|d| d.status).unwrap_or_default(),
            (active, _) if parse_boolish(active) => ApplianceStatus::Active,
            _ => ApplianceStatus::Hidden,
        };
        let description = match (slot.description.as_deref(), stored) {
            (None, Some(d)) => d.description.clone(),
            (sent, _) => normalize_description(sent.unwrap_or_default()),
        };

        next_devices.push(Appliance {
            name,
            emoji: text(&slot.emoji, |d| &d.emoji),
            description,
            featured,
            order: index as i32,
            location: text(&slot.location, |d| &d.location),
            manual_url: text(&slot.manual_url, |d| &d.manual_url),
            safety_note: text(&slot.safety_note, |d| &d.safety_note),
            status,
            model: text(&slot.model, |d| &d.model),
            video_url: text(&slot.video_url, |d| &d.video_url),
            manual_file: text(&slot.manual_file, |d| &d.manual_file),
            steps: parse_steps(slot.steps.as_ref())
                .unwrap_or_else(|| stored.map(|d| d.steps.clone()).unwrap_or_default()),
            id,
        });
    }
    Ok(next_devices)
}

/// Rien d'écrit hors du nom : la ligne qu'« Ajouter » a ouverte puis qu'on a laissée.
fn slot_is_blank(slot: &ReplaceDeviceSlot) -> bool {
    let empty = |v: &Option<String>| v.as_deref().is_none_or(|v| v.trim().is_empty());
    [
        &slot.emoji,
        &slot.location,
        &slot.manual_url,
        &slot.safety_note,
        &slot.model,
        &slot.video_url,
        &slot.manual_file,
    ]
    .into_iter()
    .all(empty)
        && slot
            .description
            .as_deref()
            .is_none_or(|d| crate::content::description_plain_text(d).trim().is_empty())
        && parse_steps(slot.steps.as_ref()).is_none_or(|steps| steps.is_empty())
}

/// Les libellés d'une `EditableList` (`[{ label }]`, en JSON ou en tableau) ou une liste de textes.
fn parse_labels(raw: &serde_json::Value) -> Vec<String> {
    use serde_json::Value;
    let parsed;
    let rows = match raw {
        Value::String(json) => {
            parsed = serde_json::from_str::<Value>(json).unwrap_or(Value::Null);
            match parsed.as_array() {
                Some(rows) => rows,
                None => return Vec::new(),
            }
        }
        Value::Array(rows) => rows,
        _ => return Vec::new(),
    };
    rows.iter()
        .filter_map(|row| match row {
            Value::String(text) => Some(text.as_str()),
            row => row.get("label").and_then(Value::as_str),
        })
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
        .collect()
}

/// Un refus dit à l'hôte dans sa langue, avec le message de la spec — pas une phrase de développeur.
fn refusal(ctx: &Context, key: &str, vars: &[(&str, &str)]) -> PortakiError {
    PortakiError::Host(
        crate::i18n::text_with(key, vars)
            .get(&ctx.locale)
            .to_string(),
    )
}

/// Les étapes saisies : les rangées de l'`EditableList` (`[{ label }]`, en JSON ou en tableau) ou
/// des textes. Les rangées vides tombent ; le nombre et la longueur se signalent sans bloquer
/// (`warnings()`). `None` : rien d'envoyé, ou rien de lisible.
fn parse_steps(raw: Option<&serde_json::Value>) -> Option<Vec<ApplianceStep>> {
    use serde_json::Value;
    let parsed;
    let rows = match raw? {
        Value::Null => return None,
        Value::String(json) if json.trim().is_empty() => return Some(Vec::new()),
        Value::String(json) => {
            parsed = serde_json::from_str::<Value>(json).ok()?;
            parsed.as_array()?
        }
        Value::Array(rows) => rows,
        _ => return None,
    };
    let field = |row: &Value, keys: &[&str]| {
        keys.iter()
            .find_map(|key| row.get(*key).and_then(Value::as_str))
            .unwrap_or_default()
            .trim()
            .to_string()
    };
    Some(
        rows.iter()
            .map(|row| match row {
                Value::String(text) => ApplianceStep {
                    text: text.trim().to_string(),
                    ..ApplianceStep::default()
                },
                row => ApplianceStep {
                    text: field(row, &["label", "text"]),
                    image: field(row, &["image"]),
                },
            })
            // Une étape vide (retirée de la liste, ou ajoutée puis laissée) tombe.
            .filter(|step| !step.text.is_empty() || !step.image.is_empty())
            .collect(),
    )
}

fn parse_boolish(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::String(s) => matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "true" | "1" | "yes" | "on"
        ),
        serde_json::Value::Number(n) => n.as_i64() == Some(1),
        _ => false,
    }
}

fn normalize_description(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string();
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if v.get("type").and_then(|t| t.as_str()) == Some("doc") {
            return trimmed.to_string();
        }
    }
    // Plain text from TextArea fallback → TipTap paragraph.
    serde_json::json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "content": [{ "type": "text", "text": trimmed }]
        }]
    })
    .to_string()
}
