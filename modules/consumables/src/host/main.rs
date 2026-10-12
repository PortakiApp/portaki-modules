//! Host dashboard surface — the settings page (spec Consommables §2), mode « page ».
//!
//! A `PropertyWorkspaceTab` surface, laid out as a settings page because it holds `Section`s, as
//! appliances does: the settings page and the « Demandes » tab of the spec are this one screen.
//!
//! Three `Section`s: the products (a table, each row opening its form in a side panel —
//! `children[i]` of the `DataTable`), the request settings, then the requests received. The first
//! two autosave through `updateConfig`; a rule broken shows under its field and blocks « Publier »
//! (`publishReadiness`), never the save. The requests sit outside that form: their reply and
//! status buttons are commands of their own.

use chrono::{DateTime, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, ChoiceList, DataTable, EmptyState, Field, FieldHint, Form, InlineNotice, List,
    NumberInput, Page, Section, Stack, Text, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};
use uuid::Uuid;

use crate::config::Stored;
use crate::entities::{ConsumableItem, ConsumableReport};
use crate::labels::{self, in_lang, labels_from_item, location_from_item};
use crate::storage::settings::{self, Settings};
use crate::{status, storage, suggest};

use super::report_ui::{build_report_block, format_relative_when, short_date};

/// Combien de produits le formulaire accepte en tout (§2.1).
pub const MAX_ITEMS: usize = 30;

/// Les produits ajoutés pas encore enregistrés : leurs id, séparés par des virgules.
const NEW_ITEMS: &str = "newItems";

/// « Demandés récemment » : une demande dans les trente derniers jours.
const RECENT_DAYS: i64 = 30;

/// Host-provided wall clock (the Wasm sandbox has none — never call `Utc::now()`).
fn host_now() -> DateTime<Utc> {
    time::now().unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

/// Réglages du module (page) : produits, demandes, puis les demandes reçues.
///
/// La nav dit au dashboard d'appeler `timelineTasks` : « Café demandé · Marie » dans À venir.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::Package
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::ConsumablesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Package
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let view = View::load(&ctx);
    let form = Form::new()
        .child(products_section(&view, &ctx))
        .child(requests_section(&view, &ctx));
    Surface::new(
        Page::new().child(
            Stack::new()
                .gap(16.0)
                .child(form)
                .child(received_section(&view, &ctx)),
        ),
    )
    .with_id(MAIN)
}

/// Ce que la page lit : l'enregistré, plus les lignes qu'« Ajouter » a ouvertes.
struct View {
    items: Vec<ConsumableItem>,
    reports: Vec<ConsumableReport>,
    settings: Settings,
    restock_delay: Option<I18nText>,
    /// Les produits ajoutés depuis l'entrée de surface, pas encore enregistrés.
    added: Vec<Uuid>,
    now: DateTime<Utc>,
}

impl View {
    fn load(ctx: &HostContext) -> Self {
        let items = storage::list_items().unwrap_or_default();
        let added = ctx
            .input_str(NEW_ITEMS)
            .unwrap_or_default()
            .split(',')
            .filter_map(|id| Uuid::parse_str(id.trim()).ok())
            .filter(|id| !items.iter().any(|item| item.id == *id))
            .collect();
        Self {
            items,
            reports: storage::list_all().unwrap_or_default(),
            settings: settings::read(),
            restock_delay: storage::restock_delay::read(),
            added,
            now: host_now(),
        }
    }

    fn row_count(&self) -> usize {
        (self.items.len() + self.added.len()).min(MAX_ITEMS.max(self.items.len()))
    }

    /// Le message sous `field`, dans la langue de l'hôte.
    fn error(&self, field: &str, ctx: &HostContext) -> Option<String> {
        Stored {
            items: &self.items,
            restock_delay: self.restock_delay.as_ref(),
            settings: &self.settings,
        }
        .error_of(field)
        .map(|error| error.get(&ctx.locale).to_string())
    }

    fn field(&self, name: &str, label: &str, ctx: &HostContext) -> Field {
        let field = Field::new().name(name).label(label);
        match self.error(name, ctx) {
            Some(error) => field.error(error),
            None => field,
        }
    }

    /// La dernière demande de ce produit, s'il y en a eu une.
    fn last_request(&self, item: Uuid) -> Option<DateTime<Utc>> {
        self.reports
            .iter()
            .filter(|report| report.item_id == item)
            .map(|report| report.created_at)
            .max()
    }
}

/// §2.1 Produits : le tableau, ses lignes, et le panneau de chacune.
fn products_section(view: &View, ctx: &HostContext) -> Component {
    let count = view.row_count();
    let mut children: Vec<Component> =
        vec![FieldHint::new().text("i18n:host.products.help").into()];
    if let Some(error) = view.error("items", ctx) {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }

    // Première ouverture, liste vide : la liste de départ (§4), en un clic.
    if count == 0 {
        children.push(
            Stack::new()
                .gap(8.0)
                .child(
                    Text::new()
                        .text("i18n:host.starter.help")
                        .variant(TextVariant::Caption),
                )
                .child(
                    Button::new()
                        .label("i18n:host.starter.add")
                        .variant(ButtonVariant::Outline)
                        .icon(IconName::Sparkles)
                        .action(
                            crate::ids::module_id().command_empty(crate::commands::SEED_DEFAULTS),
                        ),
                )
                .into(),
        );
    }

    let mut rows: Vec<Value> = Vec::with_capacity(count);
    let mut panels: Vec<Component> = Vec::with_capacity(count);
    for (index, item) in view.items.iter().enumerate() {
        let name = in_lang(&labels_from_item(item), &ctx.locale);
        let location = in_lang(&location_from_item(item), &ctx.locale);
        rows.push(row(view, item, &name, &location, ctx));
        panels.push(product_panel(
            view,
            index,
            &item.emoji,
            &name,
            &location,
            ctx,
        ));
    }
    for (offset, id) in view.added.iter().take(count - view.items.len()).enumerate() {
        rows.push(json!({ "id": id, "name": "", "location": "", "last": "" }));
        panels.push(product_panel(
            view,
            view.items.len() + offset,
            "",
            "",
            "",
            ctx,
        ));
    }

    let mut table = DataTable::new()
        .rowName("items")
        .columns(json!([
            { "key": "name", "label": "i18n:host.column.product" },
            { "key": "location", "label": "i18n:host.column.location" },
            { "key": "last", "label": "i18n:host.column.lastRequest" },
        ]))
        .rows(Value::Array(rows))
        .searchPlaceholder("i18n:host.products.search")
        .filterKey("requested")
        .children(panels);
    if count < MAX_ITEMS {
        let mut added: Vec<String> = view.added.iter().map(Uuid::to_string).collect();
        added.push(Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_ITEMS.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.products.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    } else {
        children.push(
            Text::new()
                .text("i18n:host.products.tooMany")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    children.push(table.into());

    // ponytail: l'import CSV attend que la plateforme accepte un CSV en dépôt d'hôte
    // (`HostFilePolicy` ne prend qu'images, GPX et PDF) ; le module le lirait par `files::read`.
    children.push(
        Stack::new()
            .direction(StackDirection::Horizontal)
            .gap(10.0)
            .child(
                Button::new()
                    .label("i18n:host.import.csv")
                    .variant(ButtonVariant::Outline),
            )
            .child(
                Text::new()
                    .text("i18n:host.import.soon")
                    .variant(TextVariant::Caption),
            )
            .into(),
    );

    Section::new()
        .title("i18n:host.products.title")
        .subtitle(format!("{count} / {MAX_ITEMS}"))
        .children(children)
        .into()
}

/// Produit · Où le trouver · Dernière demande, plus `requested` pour les filtres.
fn row(view: &View, item: &ConsumableItem, name: &str, location: &str, ctx: &HostContext) -> Value {
    let emoji = suggest::shown(&item.emoji, name);
    let shown = match (emoji.is_empty(), name) {
        (_, "") => String::new(),
        (true, name) => name.to_string(),
        (false, name) => format!("{emoji} {name}"),
    };
    let last = view.last_request(item.id);
    // Une demande plus ancienne n'entre dans aucun des deux filtres, seulement dans « Tous ».
    let requested = match last {
        None => "i18n:host.filter.never",
        Some(at) if (view.now - at).num_days() < RECENT_DAYS => "i18n:host.filter.recent",
        Some(_) => "",
    };
    json!({
        "id": item.id,
        "name": shown,
        "location": location,
        "last": last.map_or_else(
            || "—".to_string(),
            |at| format_relative_when(at, view.now, &ctx.locale),
        ),
        "requested": requested,
    })
}

/// Le panneau d'une ligne : emoji, nom, où le trouver, nommés `items.<i>.<clé>`.
fn product_panel(
    view: &View,
    index: usize,
    emoji: &str,
    name: &str,
    location: &str,
    ctx: &HostContext,
) -> Component {
    let key = |field: &str| format!("items.{index}.{field}");
    Stack::new()
        .gap(14.0)
        .child(
            view.field(&key("emoji"), "i18n:host.product.emoji", ctx)
                .child(
                    TextInput::new()
                        .name(key("emoji"))
                        .value(emoji)
                        .placeholder(suggest::emoji(name).unwrap_or("📦")),
                ),
        )
        .child(
            view.field(&key("label"), "i18n:host.product.name", ctx)
                .required(true)
                .child(
                    TextInput::new()
                        .name(key("label"))
                        .value(name)
                        .placeholder("i18n:host.product.name.placeholder"),
                ),
        )
        .child(
            view.field(&key("location"), "i18n:host.product.location", ctx)
                .child(
                    TextInput::new()
                        .name(key("location"))
                        .value(location)
                        .placeholder("i18n:host.product.location.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.product.location.hint")),
        )
        .into()
}

/// §2.2 Demandes. Fermées, les trois réglages se masquent (règles communes : masqué, pas grisé).
fn requests_section(view: &View, ctx: &HostContext) -> Component {
    const ON: &str = "requests_enabled=true";
    let settings = &view.settings;
    let restock_delay = view
        .restock_delay
        .as_ref()
        .map(|text| text.host_value(ctx).to_string())
        .unwrap_or_default();
    let notify = serde_json::to_string(&settings.notify()).unwrap_or_default();
    Section::new()
        .title("i18n:host.requests.title")
        .child(
            Field::new()
                .name("requests_enabled")
                .label("i18n:host.requests.enabled")
                .child(
                    Toggle::new()
                        .name("requests_enabled")
                        .checked(settings.requests_enabled()),
                )
                .child(FieldHint::new().text("i18n:host.requests.enabled.hint")),
        )
        .child(
            view.field("restock_delay", "i18n:host.main.restockDelay", ctx)
                .visibleWhen(ON)
                .child(
                    TextInput::new()
                        .name("restock_delay")
                        .value(restock_delay)
                        .placeholder("i18n:host.main.restockDelay.placeholder"),
                ),
        )
        .child(
            view.field("notify", "i18n:host.notify", ctx)
                .visibleWhen(ON)
                .required(true)
                .child(
                    ChoiceList::new()
                        .name("notify")
                        .multi(true)
                        .layout(ChoiceListLayout::Compact)
                        .choices(vec![
                            ChoiceOption::new(settings::EMAIL, "i18n:host.notify.email"),
                            ChoiceOption::new(settings::APP, "i18n:host.notify.app"),
                        ])
                        .value(notify),
                )
                // Pas de canal SMS sur la plateforme : il est annoncé, pas proposé.
                .child(FieldHint::new().text("i18n:host.notify.sms")),
        )
        .child(
            view.field("max_requests", "i18n:host.requests.max", ctx)
                .visibleWhen(ON)
                .child(
                    NumberInput::new()
                        .name("max_requests")
                        .min(f64::from(settings::MIN_MAX_REQUESTS))
                        .max(f64::from(settings::MAX_MAX_REQUESTS))
                        // Ce que l'hôte a tapé, hors bornes compris, à côté de son erreur.
                        .value(f64::from(
                            settings
                                .max_requests
                                .unwrap_or(settings::DEFAULT_MAX_REQUESTS),
                        )),
                )
                .child(FieldHint::new().text("i18n:host.requests.max.hint")),
        )
        .into()
}

/// Les demandes reçues : la table de toutes, puis celles à traiter avec leurs actions.
fn received_section(view: &View, ctx: &HostContext) -> Component {
    let locale = ctx.locale.as_str();
    let mut children: Vec<Component> = Vec::new();
    if !view.settings.requests_enabled() {
        children.push(
            InlineNotice::new()
                .message("i18n:host.received.disabled")
                .into(),
        );
    }
    let recent: Vec<&ConsumableReport> = view.reports.iter().take(40).collect();
    if recent.is_empty() {
        if view.settings.requests_enabled() {
            children.push(
                EmptyState::new()
                    .title("i18n:host.received.empty")
                    .description("i18n:host.received.empty.help")
                    .icon(IconName::Package)
                    .into(),
            );
        }
    } else {
        let emoji_of = |id: Uuid| {
            view.items
                .iter()
                .find(|item| item.id == id)
                .map(|item| {
                    suggest::shown(
                        &item.emoji,
                        &labels::pick_label(&labels_from_item(item), locale, "fr"),
                    )
                })
                .unwrap_or_default()
        };
        children.push(
            DataTable::new()
                .columns(json!([
                    { "key": "date", "label": "i18n:host.column.date" },
                    { "key": "stay", "label": "i18n:host.column.stay" },
                    { "key": "guest", "label": "i18n:host.column.guest" },
                    { "key": "product", "label": "i18n:host.column.product" },
                    { "key": "status", "label": "i18n:host.column.status", "kind": "pill" },
                ]))
                .rows(Value::Array(
                    recent
                        .iter()
                        .map(|report| {
                            let emoji = emoji_of(report.item_id);
                            json!({
                                "id": report.id,
                                "date": format_relative_when(report.created_at, view.now, locale),
                                "stay": stay_dates(report, locale),
                                "guest": if report.guest_name.trim().is_empty() { "—" } else { report.guest_name.trim() },
                                "product": format!("{emoji} {}", report.item_label).trim().to_string(),
                                "status": format!("i18n:{}", status::status_label_key(&report.status)),
                            })
                        })
                        .collect(),
                ))
                .into(),
        );
        let pending: Vec<Component> = recent
            .iter()
            .filter(|report| status::is_pending(&report.status))
            .map(|report| build_report_block(report, view.now, locale))
            .collect();
        if !pending.is_empty() {
            children.push(
                Text::new()
                    .text("i18n:host.received.toHandle")
                    .variant(TextVariant::Caption)
                    .into(),
            );
            children.push(List::new().children(pending).into());
        }
    }
    Section::new()
        .title("i18n:host.received.title")
        .children(children)
        .into()
}

/// « 12 oct. – 15 oct. », ou « — » pour une demande d'avant la colonne.
fn stay_dates(report: &ConsumableReport, locale: &str) -> String {
    match (report.stay_check_in, report.stay_check_out) {
        (Some(from), Some(to)) => {
            format!("{} – {}", short_date(from, locale), short_date(to, locale))
        }
        (Some(from), None) => short_date(from, locale),
        _ => "—".to_string(),
    }
}
