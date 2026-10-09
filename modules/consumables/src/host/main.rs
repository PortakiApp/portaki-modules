//! Host property editor — catalog + open reports (`consumables-editor-v1`).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{
    Button, Card, EmptyState, Field, FieldHint, Form, Grid, IndexedInput, InfoBanner, List,
    NumberInput, Page, Stack, Text, TextInput, Toggle,
};
use portaki_sdk::sdui::surface::Surface;

use portaki_sdk::host::time;

use crate::labels::{self, lang_code};
use crate::storage;

use super::report_ui::build_report_block;

/// Les cases que le formulaire propose d'emblée, prêtes à remplir.
///
/// La grille de huit est le dessin, et il tient : huit petites cases numérotées se lisent d'un
/// coup d'œil là où huit cartes empilées prendraient un écran. Ce qui ne tenait pas, c'est
/// qu'elle était aussi un plafond — un neuvième produit n'avait nulle part où s'écrire, et un
/// neuvième déjà stocké ne s'affichait même pas.
const ITEM_SLOTS: usize = 8;

/// Combien de produits le formulaire accepte en tout.
pub const MAX_ITEMS: usize = 30;

/// Host-provided wall clock (the Wasm sandbox has none — never call `Utc::now()`).
fn host_now() -> chrono::DateTime<chrono::Utc> {
    time::now().unwrap_or_else(|_| {
        chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).expect("epoch is valid")
    })
}

/// Host main — catalog IndexedInputs + seed defaults + open shortage reports.
///
/// Save chrome is owned by the modules sheet / workspace (`updateConfig`).
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::ConsumablesEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Package
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let lang = lang_code(&ctx.locale);
    let items = storage::list_items().unwrap_or_default();
    let open_reports = storage::list_open().unwrap_or_default();
    let locale = ctx.locale.as_str();

    let settings = storage::settings::read();
    let restock_delay = storage::restock_delay::read()
        .map(|text| text.host_value(&ctx).to_string())
        .unwrap_or_default();
    let tiles_count = draft_rows(&ctx, items.len());
    let mut tiles: Vec<Component> = Vec::with_capacity(tiles_count);
    for index in 0..tiles_count {
        let label = items
            .get(index)
            .map(|item| labels::get_label(item, &lang))
            .unwrap_or_default();

        // L'emoji à côté du nom, dans la même case : c'est ce que le voyageur touchera, et le
        // demander dans un second écran le ferait oublier.
        let emoji = items
            .get(index)
            .map(|item| item.emoji.clone())
            .unwrap_or_default();
        tiles.push(
            Stack::new()
                .direction(StackDirection::Horizontal)
                .gap(8.0)
                .child(
                    TextInput::new()
                        .name(format!("items.{index}.emoji"))
                        .value(emoji)
                        .placeholder("i18n:host.item.emoji"),
                )
                .child(
                    IndexedInput::new()
                        .index((index + 1) as u32)
                        .name(format!("items.{index}.label"))
                        .value(label)
                        .placeholder("i18n:host.item.empty")
                        .showCheck(true),
                )
                .into(),
        );
    }

    let mut catalog = Card::new()
        .title("i18n:host.main.catalogTitle")
        .subtitle("i18n:host.main.catalogHelp")
        .icon(IconName::Package)
        .child(
            Grid::new()
                .columns(4)
                .gap(10.0)
                .minColumnWidth(280.0)
                .children(tiles),
        )
        // Les demandes : ouvertes ou non, et combien par séjour (§2.2).
        .child(
            Field::new()
                .name("requests_enabled")
                .label("i18n:host.requests.enabled")
                .child(
                    Toggle::new()
                        .name("requests_enabled")
                        .checked(settings.requests_enabled()),
                ),
        )
        .child(FieldHint::new().text("i18n:host.requests.enabled.hint"));
    // Fermées, le plafond et le délai n'ont plus d'objet (règles communes : masqué, pas grisé).
    if settings.requests_enabled() {
        catalog = catalog
            .child(
                Field::new()
                    .name("max_requests")
                    .label("i18n:host.requests.max")
                    .child(
                        NumberInput::new()
                            .name("max_requests")
                            .min(f64::from(storage::settings::MIN_MAX_REQUESTS))
                            .max(f64::from(storage::settings::MAX_MAX_REQUESTS))
                            .value(f64::from(settings.max_requests())),
                    ),
            )
            .child(FieldHint::new().text("i18n:host.requests.max.hint"))
            // Ce que l'hôte promet, à côté de ce qu'il propose : le voyageur le lit avant
            // d'envoyer son signalement, et c'est ce qui lui dit que quelqu'un l'a lu.
            .child(
                Field::new()
                    .name("restock_delay")
                    .label("i18n:host.main.restockDelay")
                    .child(
                        TextInput::new()
                            .name("restock_delay")
                            .value(restock_delay)
                            .placeholder("i18n:host.main.restockDelay.placeholder"),
                    ),
            )
            .child(FieldHint::new().text("i18n:host.main.restockDelay.hint"));
    }
    // Une case de plus, quand les huit sont prises. Sous la grille et non dans un `StepList` :
    // empiler les rangées rendrait illisible ce qui se lit d'un coup d'œil en grille.
    let catalog_form = Form::new().child(
        catalog.child(
            Button::new()
                .label("i18n:host.main.addItem")
                .variant(ButtonVariant::Outline)
                .action(emit_input(RowCount {
                    items_count: (tiles_count + 1).min(MAX_ITEMS),
                })),
        ),
    );

    let seed_card = Card::new()
        .title("i18n:host.main.seedDefaults")
        .subtitle("i18n:host.main.seedHelp")
        .icon(IconName::Sparkles)
        .child(
            Form::new().child(
                Button::new()
                    .label("i18n:host.main.seedDefaults")
                    .action(crate::ids::module_id().command_empty(crate::commands::SEED_DEFAULTS)),
            ),
        );

    let recent_body: Vec<Component> = if open_reports.is_empty() {
        vec![EmptyState::new()
            .title("i18n:host.main.emptyRecent")
            .description("i18n:host.main.emptyRecent.help")
            .icon(IconName::Package)
            .into()]
    } else {
        let now = host_now();
        let report_items: Vec<Component> = open_reports
            .iter()
            .map(|report| build_report_block(report, now, locale))
            .collect();
        vec![
            Text::new()
                .text("i18n:host.main.recentIntro")
                .variant(TextVariant::Caption)
                .into(),
            Component::List(List::new().children(report_items)),
        ]
    };

    let recent_card = Card::new()
        .title("i18n:host.main.recentTitle")
        .subtitle("i18n:host.main.recentHelp")
        .icon(IconName::Package)
        .children(recent_body);

    let mut children: Vec<Component> = vec![
        InfoBanner::new().message("i18n:host.main.banner").into(),
        catalog_form.into(),
    ];
    if items.is_empty() {
        children.push(seed_card.into());
    }
    children.push(recent_card.into());

    Surface::new(Page::new().child(Stack::new().gap(16.0).children(children))).with_id(MAIN)
}

/// Combien de cases dessiner : ce que « Ajouter » a demandé, sinon les huit d'origine — ou plus
/// si l'hôte a déjà plus de produits que de cases.
fn draft_rows(ctx: &HostContext, stored: usize) -> usize {
    match ctx.input_u64("items_count") {
        Some(asked) => (asked as usize).clamp(ITEM_SLOTS, MAX_ITEMS),
        None => stored.clamp(ITEM_SLOTS, MAX_ITEMS),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct RowCount {
    items_count: usize,
}

fn emit_input(payload: impl serde::Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}
