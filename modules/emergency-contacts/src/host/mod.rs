//! Host dashboard surface — the settings page (spec Urgences §2), mode « page ».
//!
//! Three `Section`s, the shell's cards, in the spec's order: the country's numbers (read only),
//! your contacts, health. A block that depends on a choice hides with `visibleWhen` — the host's
//! number under « Afficher mon numéro », the hours under « Plages horaires ». The other contacts
//! are a sortable `DataTable` whose row opens in a dialog (`panel: "dialog"`), its fields named
//! `contacts.<i>.<key>`. A rule broken shows under its field, the same one `publishReadiness`
//! blocks on; no Save button — the shell autosaves through `updateConfig`.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{ChoiceListLayout, Tone};
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, ChoiceList, DataTable, Field, FieldHint, Form, Grid, InlineNotice, Page,
    PhoneInput, Section, Stack, Stat, TextInput, TimeRange, ToggleRow, WeeklyHours,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

pub use crate::config::MAX_CONTACTS;
use crate::config::{compact, ContactRow, ModuleConfig, AVAILABILITY};
use crate::numbers;

/// Les contacts ajoutés pas encore enregistrés : leurs id, séparés par des virgules.
const NEW_CONTACTS: &str = "newContacts";

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::EmergencyEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Phone
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    Ok(Surface::new(
        Page::new().child(
            Form::new()
                .child(country_section(&ctx))
                .child(contacts_section(&config, &ctx))
                .child(health_section(&config, &ctx)),
        ),
    )
    .with_id(MAIN))
}

// ── 2.1 Numéros du pays ──────────────────────────────────────────────────────

/// Les numéros que le livret affiche en tête, en lecture seule : le 112 d'abord.
fn country_section(ctx: &HostContext) -> Component {
    let timezone = &ctx.property.timezone;
    let help = numbers::country_of(timezone)
        .and_then(|code| {
            let country = t!(&format!("host.country.{code}")).ok()?;
            t!("host.section.country.help", country = country).ok()
        })
        .unwrap_or_else(|| "i18n:host.section.country.helpUnknown".into());
    let tiles: Vec<Component> = numbers::for_timezone(timezone)
        .iter()
        .map(|number| {
            Stat::new()
                .value(number.number)
                .label(format!("i18n:{}", number.label_key))
                .into()
        })
        .collect();
    Section::new()
        .title("i18n:host.section.country")
        .subtitle(help)
        .child(Grid::new().minColumnWidth(120.0).gap(8.0).children(tiles))
        .child(FieldHint::new().text("i18n:host.section.country.hint"))
        .into()
}

// ── 2.2 Vos contacts ─────────────────────────────────────────────────────────

fn contacts_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let host = Section::new()
        .visibleWhen("show_host=true")
        .child(
            named(config, ctx, "host_visible_phone")
                .label("i18n:host.phone.label")
                .child(
                    PhoneInput::new()
                        .name("host_visible_phone")
                        .value(config.host_visible_phone.trim())
                        .defaultCountry(default_country(ctx)),
                ),
        )
        .child(
            Field::new()
                .name("host_availability")
                .label("i18n:host.availability.label")
                .child(
                    ChoiceList::new()
                        .name("host_availability")
                        .layout(ChoiceListLayout::Segmented)
                        .value(if config.host_hours().is_some() {
                            "hours"
                        } else {
                            "always"
                        })
                        .choices(
                            AVAILABILITY
                                .iter()
                                .map(|key| {
                                    ChoiceOption::new(
                                        *key,
                                        format!("i18n:host.availability.label.{key}"),
                                    )
                                })
                                .collect(),
                        ),
                ),
        )
        .child(
            named(config, ctx, "host_hours")
                .label("i18n:host.hours.label")
                .visibleWhen("host_availability=hours")
                .child(
                    TimeRange::new()
                        .name("host_hours")
                        .value(config.host_hours_wire()),
                )
                .child(FieldHint::new().text("i18n:host.hours.hint")),
        );

    Section::new()
        .title("i18n:host.contacts.title")
        .child(
            ToggleRow::new()
                .name("show_host")
                .label("i18n:host.showHost.label")
                .description("i18n:host.showHost.hint")
                .checked(config.show_host()),
        )
        .child(host)
        .child(others(config, ctx))
        .into()
}

/// « Autres contacts » : le tableau ordonnable, « N / 20 », et la fenêtre de chaque ligne.
fn others(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let rows = rows(config, ctx);
    let host = host_number(config);
    let mut children: Vec<Component> =
        vec![FieldHint::new().text("i18n:host.contacts.hint").into()];
    if let Some(error) = config.error_of("contacts") {
        children.push(
            InlineNotice::new()
                .tone(Tone::Danger)
                .message(error.get(&ctx.locale).to_string())
                .into(),
        );
    }
    let mut table = DataTable::new()
        .rowName("contacts")
        .sortable(true)
        .panel("dialog")
        .columns(json!([
            { "key": "label", "label": "i18n:host.contact.label", "sub": "note" },
            { "key": "duplicate", "label": "", "kind": "pill" },
            { "key": "phone", "label": "i18n:host.contact.phone", "mono": true },
        ]))
        .rows(Value::Array(
            rows.iter()
                .map(|(id, contact)| row(id, contact, &host, ctx))
                .collect(),
        ))
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, (_, contact))| contact_panel(index, contact, ctx))
                .collect(),
        );
    if rows.len() < MAX_CONTACTS {
        let mut added: Vec<String> = rows
            .iter()
            .filter(|(_, c)| c.is_blank())
            .map(|(id, _)| id.clone())
            .collect();
        added.push(Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_CONTACTS.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.contacts.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    }
    children.push(table.into());
    Section::new()
        .title("i18n:host.contacts.others")
        .subtitle(format!("{} / {MAX_CONTACTS}", rows.len()))
        .children(children)
        .into()
}

/// Les lignes : les contacts enregistrés, puis ceux qu'« Ajouter » a ouverts. Un contact sans id
/// (enregistré avant le tableau) en reçoit un, stable d'un rendu à l'autre (`contact-<rang>`) :
/// la plateforme le rapproche par sa position au premier enregistrement, et l'id reste.
fn rows(config: &ModuleConfig, ctx: &HostContext) -> Vec<(String, ContactRow)> {
    // Les emplacements vides de l'ancien formulaire ne sont pas des lignes.
    let mut rows: Vec<(String, ContactRow)> = config
        .contacts
        .iter()
        .enumerate()
        .filter(|(_, contact)| !contact.is_blank())
        .map(|(index, contact)| {
            let id = match contact.id.trim() {
                "" => format!("contact-{index}"),
                id => id.to_string(),
            };
            (id, contact.clone())
        })
        .collect();
    for id in ctx.input_str(NEW_CONTACTS).unwrap_or_default().split(',') {
        let id = id.trim();
        if !id.is_empty() && !rows.iter().any(|(known, _)| known == id) {
            rows.push((id.to_string(), ContactRow::default()));
        }
    }
    rows
}

/// Le numéro de l'hôte, pour la pastille « Même numéro que vous ». Une surface hôte ne reçoit pas
/// le profil (`ctx.host` n'est rempli que côté voyageur) : seul le numéro saisi ici compte.
fn host_number(config: &ModuleConfig) -> String {
    if config.show_host() {
        compact(config.host_visible_phone.trim())
    } else {
        String::new()
    }
}

/// Une ligne du tableau : nom (« Quand l'appeler » dessous), la pastille de doublon, le numéro.
fn row(id: &str, contact: &ContactRow, host: &str, ctx: &HostContext) -> Value {
    let phone = contact.phone.trim();
    let duplicate = !host.is_empty() && compact(phone) == host;
    json!({
        "id": id,
        "label": contact.label.host_value(ctx),
        "note": contact.note.host_value(ctx),
        "duplicate": if duplicate { "i18n:host.contact.duplicate" } else { "" },
        "phone": phone,
    })
}

/// La fenêtre d'une ligne : Nom ou rôle, Téléphone, Quand l'appeler (§2.2).
fn contact_panel(index: usize, contact: &ContactRow, ctx: &HostContext) -> Component {
    let name = |key: &str| format!("contacts.{index}.{key}");
    let problems = contact.problems();
    let field = |key: &str| {
        let field = Field::new().name(name(key));
        match problems.iter().find(|(name, _)| *name == key) {
            Some((_, error)) => field.error(error.get(&ctx.locale).to_string()),
            None => field,
        }
    };
    Stack::new()
        .gap(14.0)
        .children(vec![
            field("label")
                .label("i18n:host.contact.label")
                .required(true)
                .child(
                    TextInput::new()
                        .name(name("label"))
                        .value(contact.label.host_value(ctx))
                        .placeholder("i18n:host.contact.label.placeholder"),
                )
                .into(),
            field("phone")
                .label("i18n:host.contact.phone")
                .required(true)
                .child(
                    PhoneInput::new()
                        .name(name("phone"))
                        .value(contact.phone.trim())
                        .defaultCountry(default_country(ctx)),
                )
                .into(),
            field("note")
                .label("i18n:host.contact.note")
                .child(
                    TextInput::new()
                        .name(name("note"))
                        .value(contact.note.host_value(ctx))
                        .placeholder("i18n:host.contact.note.placeholder"),
                )
                .into(),
        ])
        .into()
}

// ── 2.3 Santé ────────────────────────────────────────────────────────────────

fn health_section(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let pharmacy = place(
        config,
        ctx,
        "pharmacy",
        (&config.pharmacy, &config.pharmacy_phone),
        (
            &config.pharmacy_address,
            config.pharmacy_lat,
            config.pharmacy_lng,
        ),
    )
    .child(
        Field::new()
            .name("pharmacy_hours")
            .label("i18n:host.pharmacyHours.label")
            .child(
                WeeklyHours::new()
                    .name("pharmacy_hours")
                    .value(config.pharmacy_hours.trim()),
            ),
    );
    let mut children: Vec<Component> = vec![pharmacy.into()];
    // La pharmacie de garde : lecture seule, le numéro du pays (le 3237 en France), rien ailleurs.
    let timezone = &ctx.property.timezone;
    if let (Some(number), Some(code)) = (
        numbers::on_duty_pharmacy(timezone),
        numbers::country_of(timezone),
    ) {
        if let Some(line) = t!(&format!("host.country.{code}"))
            .ok()
            .and_then(|country| t!("host.onDuty", number = number, country = country).ok())
        {
            children.push(InlineNotice::new().message(line).into());
        }
    }
    children.push(
        place(
            config,
            ctx,
            "hospital",
            (&config.hospital, &config.hospital_phone),
            (
                &config.hospital_address,
                config.hospital_lat,
                config.hospital_lng,
            ),
        )
        .into(),
    );
    children.push(
        place(
            config,
            ctx,
            "doctor",
            (&config.doctor, &config.doctor_phone),
            (&config.doctor_address, config.doctor_lat, config.doctor_lng),
        )
        .into(),
    );
    Section::new()
        .title("i18n:host.section.health")
        .subtitle("i18n:host.section.health.help")
        .children(children)
        .into()
}

/// Un lieu de santé : nom, téléphone, position. Ses champs s'appellent `{key}`, `{key}_phone`,
/// `{key}_address` / `_lat` / `_lng`.
fn place(
    config: &ModuleConfig,
    ctx: &HostContext,
    key: &str,
    (name, phone): (&str, &str),
    (address, lat, lng): (&str, Option<f64>, Option<f64>),
) -> Section {
    let phone_key = format!("{key}_phone");
    let mut picker = AddressMapPicker::new()
        .addressName(format!("{key}_address"))
        .latName(format!("{key}_lat"))
        .lngName(format!("{key}_lng"))
        .address(address)
        .label("i18n:host.place.position");
    if let (Some(lat), Some(lng)) = (lat, lng) {
        picker = picker.lat(lat).lng(lng);
    }
    Section::new()
        .title(format!("i18n:host.{key}.label"))
        .child(
            Field::new().name(key).label("i18n:host.place.name").child(
                TextInput::new()
                    .name(key)
                    .value(name)
                    .placeholder(format!("i18n:host.{key}.placeholder")),
            ),
        )
        .child(
            named(config, ctx, phone_key.clone())
                .label("i18n:host.place.phone")
                .child(
                    PhoneInput::new()
                        .name(phone_key)
                        .value(phone.trim())
                        .defaultCountry(default_country(ctx)),
                ),
        )
        .child(picker)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Le champ `name`, avec le message de [`ModuleConfig::error_of`] sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: impl Into<String>) -> Field {
    let name = name.into();
    let field = Field::new().name(name.clone());
    match config.error_of(&name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// L'indicatif proposé d'abord : celui du pays du logement, la France sinon.
fn default_country(ctx: &HostContext) -> &'static str {
    numbers::country_of(&ctx.property.timezone).unwrap_or("FR")
}
