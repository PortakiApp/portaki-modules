//! Host dashboard surface — the settings page (spec Appareils §2), mode « page ».
//!
//! A `PropertyWorkspaceTab` surface: the dashboard opens it as the listing tab
//! `/listings/[id]/appliances`, laid out as a settings page because it holds `Section`s. So the
//! settings page and the « Appareils » tab of the spec are this one screen, with one table.
//!
//! Two `Section`s — the shell's two cards — in the spec's order: the home card, then the
//! appliances. The appliances are a table (règles communes §2.4 bis): each row opens its form in a
//! side panel drawn by the dashboard, `children[i]` of the `DataTable`. Every edit autosaves through
//! `updateConfig`; a rule broken shows under its field and blocks « Publier »
//! (`publishReadiness`), never the save.

mod stats;

pub use stats::{render_host_stats, stats_summary};

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, DataTable, EditableList, Field, FieldHint, FileUpload, Form, ImageUpload, InlineNotice,
    NumberInput, Page, RichTextEditor, Section, Stack, StepList, Text, TextArea, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;
use portaki_sdk::sdui::EditableListItem;
use serde_json::{json, Map, Value};

use crate::content::{
    Appliance, ApplianceStatus, AppliancesBundle, AppliancesPayload, MAX_APPLIANCES, MAX_FEATURED,
    MAX_STEPS, MIN_FEATURED,
};
use crate::{store, suggest};

/// « Notice PDF » : 20 Mo au plus (§2.2).
const MANUAL_MAX_BYTES: u32 = 20 * 1024 * 1024;
/// Les appareils ajoutés pas encore enregistrés : leurs id, séparés par des virgules.
const NEW_DEVICES: &str = "newDevices";

/// Réglages du module (page) : carte d'accueil, puis appareils.
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::AppliancesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Plug
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let view = View::load(&ctx);
    let form = Form::new()
        .child(home_card_section(&view, &ctx))
        .child(devices_section(&view, &ctx));
    Surface::new(Page::new().child(form)).with_id(MAIN)
}

/// Ce que la page lit : l'enregistré, plus les lignes qu'« Ajouter » a ouvertes.
struct View {
    payload: AppliancesPayload,
    bundle: AppliancesBundle,
    /// Les appareils ajoutés depuis l'entrée de surface, pas encore enregistrés.
    added: Vec<String>,
}

impl View {
    fn load(ctx: &HostContext) -> Self {
        let bundle = store::load_bundle().unwrap_or_default();
        let payload = bundle.for_host(&ctx.locale, &ctx.property.locale);
        let added = ctx
            .input_str(NEW_DEVICES)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty() && payload.find_device(id).is_none())
            .map(str::to_string)
            .collect();
        Self {
            payload,
            bundle,
            added,
        }
    }

    /// Les lignes du tableau : les appareils enregistrés, puis les ajoutés (bornés à 60).
    fn rows(&self) -> Vec<Appliance> {
        let mut rows = self.payload.devices.clone();
        for id in &self.added {
            if rows.len() >= MAX_APPLIANCES {
                break;
            }
            rows.push(Appliance {
                id: id.clone(),
                ..Appliance::default()
            });
        }
        rows
    }

    /// Le message sous `field` (`devices.<i>.<clé>`, `featuredLimit`…), dans la langue de l'hôte.
    fn error(&self, field: &str, ctx: &HostContext) -> Option<String> {
        self.payload
            .problems_in(&self.bundle)
            .into_iter()
            .chain(self.payload.warnings())
            .find(|(name, _)| name == field)
            .map(|(_, error)| error.get(&ctx.locale).to_string())
    }

    fn field(&self, name: &str, label: &str, ctx: &HostContext) -> Field {
        let field = Field::new().name(name).label(label);
        match self.error(name, ctx) {
            Some(error) => field.error(error),
            None => field,
        }
    }
}

/// §2.1 Carte d'accueil.
fn home_card_section(view: &View, ctx: &HostContext) -> Component {
    let payload = &view.payload;
    // La valeur saisie, même hors bornes : « Entre 2 et 6. » sous un 6 bien sage serait un faux
    // positif.
    let featured_limit = payload
        .featured_limit
        .map_or(payload.featured_limit(), |n| n as usize);
    let mut children: Vec<Component> = vec![view
        .field("featuredLimit", "i18n:host.featured.label", ctx)
        .required(true)
        .child(
            NumberInput::new()
                .name("featuredLimit")
                .min(MIN_FEATURED as f64)
                .max(MAX_FEATURED as f64)
                .value(featured_limit as f64),
        )
        .child(FieldHint::new().text("i18n:host.featured.hint"))
        .into()];

    // Rien à ordonner sous deux pièces.
    let rooms = payload.rooms();
    if rooms.len() > 1 {
        children.push(
            Field::new()
                .name("roomOrder")
                .label("i18n:host.roomOrder.label")
                .child(
                    EditableList::new()
                        .name("roomOrder")
                        .items(rooms.iter().map(EditableListItem::new).collect()),
                )
                .child(FieldHint::new().text("i18n:host.roomOrder.hint"))
                .into(),
        );
    }

    children.push(
        view.field("paperManualsLocation", "i18n:host.paperManuals.label", ctx)
            .child(
                TextInput::new()
                    .name("paperManualsLocation")
                    .value(payload.paper_manuals_location.as_str())
                    .placeholder("i18n:host.paperManuals.placeholder"),
            )
            .into(),
    );

    Section::new()
        .title("i18n:host.section.homeCard")
        .children(children)
        .into()
}

/// §2.2 Appareils : le tableau, ses lignes, et le panneau de chacune.
fn devices_section(view: &View, ctx: &HostContext) -> Component {
    let rows = view.rows();
    let limit = view.payload.featured_limit();
    let mut children: Vec<Component> = Vec::new();

    if !rows.is_empty() && !rows.iter().any(|d| d.featured) {
        children.push(FieldHint::new().text("i18n:host.featured.none").into());
    }
    if let Some(error) = view.error("devices", ctx) {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }

    let mut table = DataTable::new()
        .rowName("devices")
        .columns(columns(limit))
        .rows(Value::Array(rows.iter().map(row).collect()))
        .searchPlaceholder("i18n:host.search.placeholder")
        .filterKey("room")
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, device)| device_panel(view, index, device, ctx))
                .collect(),
        );
    if rows.len() < MAX_APPLIANCES {
        let mut added = view.added.clone();
        added.push(uuid::Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_DEVICES.into(), Value::String(added.join(",")));
        table = table.addLabel("i18n:host.list.add").addAction(Action::emit(
            contracts::shell::SURFACE_INPUT,
            Some(Value::Object(input)),
        ));
    } else {
        children.push(
            Text::new()
                .text("i18n:host.devices.tooMany")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    children.push(table.into());

    // ponytail: l'import CSV attend que la plateforme accepte un CSV en dépôt d'hôte (`HostFilePolicy`
    // ne prend qu'images, GPX et PDF) ; le module le lirait par `files::read`.
    children.push(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(10.0)
            .child(Button::new().label("i18n:host.import.csv"))
            .child(
                Text::new()
                    .text("i18n:host.import.soon")
                    .variant(TextVariant::Caption),
            )
            .into(),
    );

    Section::new()
        .title("i18n:host.list.title")
        .subtitle(format!("{} / {MAX_APPLIANCES}", rows.len()))
        .children(children)
        .into()
}

/// Appareil · Pièce · Mis en avant · Actif · Mode d'emploi.
fn columns(featured_limit: usize) -> Value {
    let count = featured_limit.to_string();
    let too_many = t!("host.featured.tooMany", count = &count)
        .ok()
        .filter(|text| text.contains(&count))
        .unwrap_or_else(|| "i18n:host.featured.label".into());
    json!([
        { "key": "name", "label": "i18n:host.column.device" },
        { "key": "room", "label": "i18n:host.column.room" },
        { "key": "featured", "label": "i18n:host.column.featured", "kind": "check",
          "max": featured_limit, "maxMessage": too_many },
        { "key": "active", "label": "i18n:host.column.active", "kind": "check" },
        { "key": "manual", "label": "i18n:host.column.manual", "kind": "pill" },
    ])
}

/// Une ligne du tableau. `featured` et `active` sont aussi les clés du formulaire de la ligne.
fn row(device: &Appliance) -> Value {
    let emoji = device.shown_emoji();
    let name = match (emoji.is_empty(), device.name.trim()) {
        (_, "") => String::new(),
        (true, name) => name.to_string(),
        (false, name) => format!("{emoji} {name}"),
    };
    let room = match device.location.trim() {
        "" => "i18n:explore.detail.room.other".to_string(),
        room => room.to_string(),
    };
    // Un appareil tout juste ajouté n'a pas encore de mode d'emploi à réclamer.
    let manual = if device.has_manual() || device.name.trim().is_empty() {
        ""
    } else {
        "i18n:host.manual.missing"
    };
    json!({
        "id": device.id,
        "name": name,
        "room": room,
        "featured": device.featured,
        "active": device.status == ApplianceStatus::Active,
        "manual": manual,
    })
}

/// Le panneau d'une ligne : tous les champs de l'appareil, nommés `devices.<i>.<clé>`.
fn device_panel(view: &View, index: usize, device: &Appliance, ctx: &HostContext) -> Component {
    let name = |key: &str| format!("devices.{index}.{key}");
    let field = |key: &str, label: &str| view.field(&name(key), label, ctx);

    let emoji_placeholder = suggest::emoji(&device.name).unwrap_or("🔌");
    let room_placeholder = suggest::room_key(&device.name)
        .and_then(|key| t!(key).ok())
        .unwrap_or_else(|| "i18n:host.device.location.placeholder".into());

    Stack::new()
        .gap(14.0)
        .children(vec![
            field("name", "i18n:host.device.name")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("name"))
                        .value(device.name.as_str())
                        .placeholder("i18n:host.device.name.placeholder"),
                )
                .into(),
            field("emoji", "i18n:host.device.emoji")
                .child(
                    TextInput::new()
                        .name(name("emoji"))
                        .value(device.emoji.as_str())
                        .placeholder(emoji_placeholder),
                )
                .into(),
            field("location", "i18n:host.device.location")
                .child(
                    TextInput::new()
                        .name(name("location"))
                        .value(device.location.as_str())
                        .placeholder(room_placeholder),
                )
                .into(),
            field("model", "i18n:host.device.model")
                .child(
                    TextInput::new()
                        .name(name("model"))
                        .value(device.model.as_str())
                        .placeholder("Bosch Serie 6 WAU28"),
                )
                .into(),
            field("featured", "i18n:host.device.featured")
                .child(
                    Toggle::new()
                        .name(name("featured"))
                        .checked(device.featured),
                )
                .into(),
            field("active", "i18n:host.device.active")
                .child(
                    Toggle::new()
                        .name(name("active"))
                        .checked(device.status == ApplianceStatus::Active),
                )
                .child(FieldHint::new().text("i18n:host.device.active.hint"))
                .into(),
            steps_field(view, index, device, ctx),
            field("safetyNote", "i18n:host.device.safetyNote")
                .child(
                    TextArea::new()
                        .name(name("safetyNote"))
                        .value(device.safety_note.as_str())
                        .placeholder("i18n:host.device.safetyNote.placeholder"),
                )
                .into(),
            field("manualFile", "i18n:host.device.manualFile")
                .child(
                    FileUpload::new()
                        .name(name("manualFile"))
                        .value(device.manual_file.as_str())
                        .accept("application/pdf")
                        .maxBytes(MANUAL_MAX_BYTES),
                )
                .into(),
            field("manualUrl", "i18n:host.device.manualUrl")
                .child(
                    TextInput::new()
                        .name(name("manualUrl"))
                        .value(device.manual_url.as_str())
                        .placeholder("https://…"),
                )
                .into(),
            field("videoUrl", "i18n:host.device.videoUrl")
                .child(
                    TextInput::new()
                        .name(name("videoUrl"))
                        .value(device.video_url.as_str())
                        .placeholder("https://…"),
                )
                .into(),
            field("description", "i18n:host.device.description")
                .child(
                    RichTextEditor::new()
                        .name(name("description"))
                        .value(editor_value(&device.description)),
                )
                .child(FieldHint::new().text("i18n:host.device.description.hint"))
                .into(),
        ])
        .into()
}

/// « Étapes » : texte et schéma par étape, 15 au plus. « Ajouter une étape » rend une rangée de
/// plus ; une rangée laissée vide tombe à l'enregistrement.
fn steps_field(view: &View, index: usize, device: &Appliance, ctx: &HostContext) -> Component {
    let prefix = format!("devices.{index}.steps");
    let count_key = format!("steps.{}", device.id);
    let count = ctx
        .input_u64(&count_key)
        .map(|n| n as usize)
        .unwrap_or(0)
        .max(device.steps.len())
        .min(MAX_STEPS);
    let rows: Vec<Component> = (0..count)
        .map(|n| {
            let step = device.steps.get(n).cloned().unwrap_or_default();
            Stack::new()
                .gap(8.0)
                .child(
                    TextArea::new()
                        .name(format!("{prefix}.{n}.text"))
                        .value(step.text)
                        .rows(2),
                )
                .child(
                    ImageUpload::new()
                        .name(format!("{prefix}.{n}.image"))
                        .value(step.image),
                )
                .into()
        })
        .collect();
    let mut list = StepList::new()
        .itemKeyPrefix(prefix)
        .hint("i18n:host.device.steps.hint")
        .addLabel("i18n:host.device.steps.add")
        .children(rows);
    let mut children: Vec<Component> = Vec::new();
    if count < MAX_STEPS {
        let mut input = Map::new();
        input.insert(count_key, Value::from(count + 1));
        list = list.addAction(Action::emit(
            contracts::shell::SURFACE_INPUT,
            Some(Value::Object(input)),
        ));
    }
    children.push(list.into());
    if count >= MAX_STEPS {
        children.push(
            Text::new()
                .text("i18n:host.device.steps.tooMany")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    view.field(
        &format!("devices.{index}.steps"),
        "i18n:host.device.steps",
        ctx,
    )
    .children(children)
    .into()
}

fn editor_value(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string();
    }
    // Already TipTap JSON — keep as-is; plain text is wrapped by the shell parser.
    trimmed.to_string()
}
