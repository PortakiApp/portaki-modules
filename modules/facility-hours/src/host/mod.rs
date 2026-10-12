//! Host dashboard surface — the settings page (spec Horaires §2), mode « page ».
//!
//! A `PropertyWorkspaceTab` surface: the dashboard lays it out as a settings page because it holds
//! `Section`s. Two of them, in the spec's order: « Affichage », then « Équipements ».
//!
//! The facilities are a table (règles communes §2.4 bis), the way `appliances` draws its devices:
//! each row opens its form in a side panel, `children[i]` of the `DataTable`, its fields named
//! `facilities.<i>.<clé>`. The dashboard sends each row's `id` itself; `updateConfig` merges a row
//! that only sends its id, so a row never opened keeps everything.
//!
//! The blocks that depend on the mode hide with `Field.visibleWhen` on the row's `mode`: the
//! `RadioGroup` seeds its value, so the condition holds from the first render.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::hours::format_week;
use portaki_sdk::sdui::primitives::{
    ChoiceList, DataTable, DatePicker, Field, FieldHint, Form, InlineNotice, NumberInput, Page,
    RadioGroup, Section, Select, Stack, StepList, Text, TextArea, TextInput, TimeRange, Toggle,
    WeeklyHours,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Map, Value};

pub use crate::config::MAX_FACILITIES;
use crate::config::{
    suggest_icon, FacilityRow, ModuleConfig, GROUPS, ICONS, MAX_CARD_LIMIT, MAX_EXCEPTIONS,
    MAX_RANGES, MIN_CARD_LIMIT, MODES, MODE_ALWAYS, MODE_BY_DAY, MODE_ON_REQUEST, MODE_SAME,
};
use crate::schedule::{local_time, season_display, state_label};

/// Les équipements ajoutés pas encore enregistrés : leurs id, séparés par des virgules.
const NEW_FACILITIES: &str = "newFacilities";
/// La plage d'un équipement tout juste ajouté (§2.2, « défaut 9:00 – 19:00 »).
const DEFAULT_RANGE: &str = "09:00-19:00";

/// Réglages du module (page) : affichage, puis équipements.
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::FacilityEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::ClockCircle
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let view = View::load(&ctx)?;
    let form = Form::new()
        .child(display_section(&view, &ctx))
        .child(facilities_section(&view, &ctx));
    Ok(Surface::new(Page::new().child(form)).with_id(MAIN))
}

/// Ce que la page lit : l'enregistré, les lignes qu'« Ajouter » a ouvertes, et l'heure du lieu.
struct View {
    config: ModuleConfig,
    /// Les équipements ajoutés depuis l'entrée de surface, pas encore enregistrés.
    added: Vec<String>,
    /// Les erreurs, puis les avertissements, par champ (`facilities.<i stocké>.<clé>`).
    messages: Vec<(String, I18nText)>,
    now: Option<DateTime<Utc>>,
    tz: Option<PropertyTz>,
}

/// Une ligne du tableau : sa place dans le formulaire, et dans la config stockée.
struct Row<'a> {
    /// L'index du formulaire, `facilities.<index>.…`.
    index: usize,
    /// L'index stocké, celui des messages ; `None` pour une ligne ajoutée.
    stored: Option<usize>,
    facility: &'a FacilityRow,
}

impl View {
    fn load(ctx: &HostContext) -> Result<Self> {
        let config = ModuleConfig::load(ctx)?;
        let tz = PropertyTz::parse(&ctx.property.timezone);
        // Sans horloge (un aperçu), la colonne « Aujourd'hui » reste vide plutôt que d'inventer.
        let now = time::now().ok();
        let today = now.map(|now| local_time(now, tz.as_ref()).date());
        let messages = config
            .problems()
            .into_iter()
            .chain(config.warnings(today))
            .collect();
        let added = ctx
            .input_str(NEW_FACILITIES)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty() && !config.facilities.iter().any(|f| f.id == *id))
            .map(str::to_string)
            .collect();
        Ok(Self {
            config,
            added,
            messages,
            now,
            tz,
        })
    }

    /// Les lignes du tableau : les équipements enregistrés, sans les emplacements vides de l'ancien
    /// formulaire, puis les ajoutés (30 au plus). Une ligne d'avant les id en reçoit un fixe : le
    /// tableau écarte une ligne sans id, et la plateforme retombe alors sur la position.
    fn rows(&self) -> Vec<(Option<usize>, FacilityRow)> {
        let mut rows: Vec<(Option<usize>, FacilityRow)> = self
            .config
            .facilities
            .iter()
            .enumerate()
            .filter(|(_, facility)| !facility.is_blank())
            .map(|(stored, facility)| {
                let mut facility = facility.clone();
                if facility.id.trim().is_empty() {
                    facility.id = format!("legacy-{stored}");
                }
                (Some(stored), facility)
            })
            .collect();
        for id in &self.added {
            if rows.len() >= MAX_FACILITIES {
                break;
            }
            rows.push((
                None,
                FacilityRow {
                    id: id.clone(),
                    ..FacilityRow::default()
                },
            ));
        }
        rows
    }

    fn message(&self, key: &str, ctx: &HostContext) -> Option<String> {
        self.messages
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, text)| text.get(&ctx.locale).to_string())
    }

    /// Le champ `name`, libellé, avec le message de `key` (l'index stocké) sous lui.
    fn field(&self, name: &str, key: Option<&str>, label: &str, ctx: &HostContext) -> Field {
        let field = Field::new().name(name).label(label);
        match key.and_then(|key| self.message(key, ctx)) {
            Some(message) => field.error(message),
            None => field,
        }
    }
}

impl Row<'_> {
    fn name(&self, key: &str) -> String {
        format!("facilities.{}.{key}", self.index)
    }

    fn field(&self, view: &View, key: &str, label: &str, ctx: &HostContext) -> Field {
        let stored = self
            .stored
            .map(|stored| format!("facilities.{stored}.{key}"));
        view.field(&self.name(key), stored.as_deref(), label, ctx)
    }

    /// `visibleWhen` du mode `mode`.
    fn when(&self, mode: &str) -> String {
        format!("{}={mode}", self.name("mode"))
    }
}

/// §2.1 Affichage.
fn display_section(view: &View, ctx: &HostContext) -> Component {
    let config = &view.config;
    // La valeur saisie, même hors bornes : « Entre 1 et 6. » sous un 3 bien sage serait un faux
    // positif.
    let shown = if config.card_limit.is_finite() && config.card_limit != 0.0 {
        config.card_limit
    } else {
        config.card_limit() as f64
    };
    let mut children: Vec<Component> = vec![view
        .field(
            "card_limit",
            Some("card_limit"),
            "i18n:host.cardLimit.label",
            ctx,
        )
        .required(true)
        .child(
            NumberInput::new()
                .name("card_limit")
                .min(MIN_CARD_LIMIT as f64)
                .max(MAX_CARD_LIMIT as f64)
                .value(shown),
        )
        .child(FieldHint::new().text("i18n:host.cardLimit.hint"))
        .into()];
    // L'ancienne note générale, hors spec : montrée tant qu'elle porte un texte, pour que l'hôte
    // puisse la vider — le livret la lit encore.
    let note = config.general_note.host_value(ctx);
    if !note.trim().is_empty() {
        children.push(
            Field::new()
                .name("general_note")
                .label("i18n:host.note.label")
                .child(
                    TextArea::new()
                        .name("general_note")
                        .value(note)
                        .placeholder("i18n:host.note.placeholder"),
                )
                .into(),
        );
    }
    Section::new()
        .title("i18n:host.display.title")
        .children(children)
        .into()
}

/// §2.2 Équipements : le tableau, ses lignes, et le panneau de chacune.
fn facilities_section(view: &View, ctx: &HostContext) -> Component {
    let rows = view.rows();
    let mut children: Vec<Component> = Vec::new();
    if let Some(error) = view.message("facilities", ctx) {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }

    let mut table = DataTable::new()
        .rowName("facilities")
        .columns(json!([
            { "key": "name", "label": "i18n:host.column.facility" },
            { "key": "group", "label": "i18n:host.column.group" },
            { "key": "hours", "label": "i18n:host.column.hours" },
            { "key": "today", "label": "i18n:host.column.today", "kind": "pill" },
        ]))
        .rows(Value::Array(
            rows.iter()
                .map(|(_, facility)| table_row(view, facility, ctx))
                .collect(),
        ))
        .searchPlaceholder("i18n:host.search.placeholder")
        .filterKey("group")
        .children(
            rows.iter()
                .enumerate()
                .map(|(index, (stored, facility))| {
                    let row = Row {
                        index,
                        stored: *stored,
                        facility,
                    };
                    facility_panel(view, &row, ctx)
                })
                .collect(),
        );
    if rows.len() < MAX_FACILITIES {
        let mut added = view.added.clone();
        added.push(Uuid::new_v4().to_string());
        let mut input = Map::new();
        input.insert(NEW_FACILITIES.into(), Value::String(added.join(",")));
        table = table
            .addLabel("i18n:host.facilities.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(Value::Object(input)),
            ));
    } else {
        children.push(
            Text::new()
                .text("i18n:host.facilities.tooMany")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    children.push(table.into());

    Section::new()
        .title("i18n:host.facilities.title")
        .subtitle(format!("{} / {MAX_FACILITIES}", rows.len()))
        .children(children)
        .into()
}

/// Équipement · Groupe · Horaires · Aujourd'hui.
fn table_row(view: &View, facility: &FacilityRow, ctx: &HostContext) -> Value {
    json!({
        "id": facility.id,
        "name": facility.title.host_value(ctx),
        "group": format!("i18n:host.facility.group.{}", facility.group_key()),
        "hours": hours_summary(facility),
        "today": today_label(view, facility, ctx),
    })
}

/// Les horaires d'une ligne en une cellule : les plages en mono, ou ce que le mode dit.
fn hours_summary(facility: &FacilityRow) -> String {
    match facility.mode() {
        MODE_BY_DAY => "i18n:host.facility.mode.by_day".to_string(),
        MODE_ALWAYS => "i18n:host.facility.mode.always".to_string(),
        MODE_ON_REQUEST => "i18n:host.facility.mode.on_request".to_string(),
        _ => {
            let ranges = facility.same_ranges();
            if ranges.is_empty() {
                facility.hours.as_deref().unwrap_or("").trim().to_string()
            } else {
                ranges
                    .iter()
                    .map(|(start, end)| format!("{start} – {end}"))
                    .collect::<Vec<_>>()
                    .join(" · ")
            }
        }
    }
}

/// « Ouvert · ferme à 19:00 », « Ferme dans 40 min », « Fermé », « 24 h/24 », « Sur demande »,
/// « Fermé jusqu'au 1er juin » — au fuseau du logement, jamais à celui du navigateur.
fn today_label(view: &View, facility: &FacilityRow, ctx: &HostContext) -> String {
    let state = view
        .now
        .and_then(|now| facility.schedule().state_at(now, view.tz.as_ref()));
    match state {
        Some(state) => state_label(&state, &ctx.locale)
            .map(|(label, _)| label)
            .unwrap_or_else(|| "i18n:host.facility.mode.always".to_string()),
        None if facility.mode() == MODE_ON_REQUEST => {
            "i18n:host.facility.mode.on_request".to_string()
        }
        None => String::new(),
    }
}

/// Le panneau d'une ligne : tous ses champs, nommés `facilities.<i>.<clé>`.
fn facility_panel(view: &View, row: &Row, ctx: &HostContext) -> Component {
    let facility = row.facility;
    let title = facility.title.host_value(ctx);
    let mode = facility.mode();
    // Jamais choisie : celle que le nom appelle (§4). « Aucune » enregistrée reste aucune.
    let icon = match &facility.icon {
        Some(icon) => icon.trim().to_string(),
        None => suggest_icon(title).unwrap_or("").to_string(),
    };

    let mut children: Vec<Component> = vec![
        row.field(view, "icon", "i18n:host.facility.icon", ctx)
            .child(
                Select::new()
                    .name(row.name("icon"))
                    .options(
                        std::iter::once(ChoiceOption::new("", "i18n:host.facility.icon.none"))
                            .chain(ICONS.iter().map(|icon| {
                                ChoiceOption::new(*icon, format!("i18n:host.icon.{icon}"))
                            }))
                            .collect(),
                    )
                    .value(icon),
            )
            .into(),
        row.field(view, "title", "i18n:host.facility.name", ctx)
            .required(true)
            .child(
                TextInput::new()
                    .name(row.name("title"))
                    .value(title)
                    .placeholder("i18n:host.facility.name.placeholder"),
            )
            .into(),
        row.field(view, "group", "i18n:host.facility.group", ctx)
            .required(true)
            .child(
                RadioGroup::new()
                    .name(row.name("group"))
                    .options(
                        GROUPS
                            .iter()
                            .map(|key| {
                                ChoiceOption::new(*key, format!("i18n:host.facility.group.{key}"))
                            })
                            .collect(),
                    )
                    .value(facility.group_key()),
            )
            .into(),
        row.field(view, "mode", "i18n:host.facility.mode", ctx)
            .required(true)
            .child(
                RadioGroup::new()
                    .name(row.name("mode"))
                    .options(
                        MODES
                            .iter()
                            .map(|key| {
                                ChoiceOption::new(*key, format!("i18n:host.facility.mode.{key}"))
                            })
                            .collect(),
                    )
                    .value(mode),
            )
            .into(),
    ];
    children.extend(same_ranges_fields(view, row, ctx));
    children.push(week_field(view, row, ctx));
    children.push(note_field(view, row, ctx));
    children.push(closed_days_field(row));
    children.extend(season_fields(view, row, ctx));
    children.push(exceptions_field(view, row, ctx));
    children.extend(legacy_fields(view, row, ctx));

    Stack::new().gap(14.0).children(children).into()
}

/// « Même horaire tous les jours » : trois plages, la première remplie. Une ligne d'avant les
/// plages montre son ouverture, sa fermeture et sa coupure, migrées.
fn same_ranges_fields(view: &View, row: &Row, ctx: &HostContext) -> Vec<Component> {
    let facility = row.facility;
    let fields = facility.range_fields();
    let typed = fields.iter().any(|(_, raw)| !raw.trim().is_empty());
    let migrated = facility.same_ranges();
    let added = row.stored.is_none();
    (0..MAX_RANGES)
        .map(|k| {
            let (key, raw) = fields[k];
            let value = if typed {
                raw.trim().to_string()
            } else if let Some((start, end)) = migrated.get(k) {
                format!("{start}-{end}")
            } else if k == 0 && added {
                DEFAULT_RANGE.to_string()
            } else {
                String::new()
            };
            let label = format!("i18n:host.facility.range.{}", k + 1);
            let mut field = row
                .field(view, key, &label, ctx)
                .visibleWhen(row.when(MODE_SAME))
                .child(TimeRange::new().name(row.name(key)).value(value));
            if k == 0 {
                field = field.child(FieldHint::new().text("i18n:host.facility.ranges.hint"));
            }
            field.into()
        })
        .collect()
}

/// « Selon le jour » : la semaine, trois plages par jour au plus.
fn week_field(view: &View, row: &Row, ctx: &HostContext) -> Component {
    let facility = row.facility;
    let week = match facility.week.trim() {
        "" => format_week(&facility.week_ranges()),
        week => week.to_string(),
    };
    // ponytail: « Copier le lundi sur toute la semaine » attend le dashboard — une valeur déjà
    // semée ne se remplace pas depuis le module (`useSduiValue` garde la valeur du formulaire).
    row.field(view, "week", "i18n:host.facility.week", ctx)
        .visibleWhen(row.when(MODE_BY_DAY))
        .child(
            WeeklyHours::new()
                .name(row.name("week"))
                .value(week)
                .maxRanges(MAX_RANGES as u32),
        )
        .child(FieldHint::new().text("i18n:host.facility.week.hint"))
        .into()
}

/// « Comment réserver » : obligatoire sur demande, masqué ailleurs — sauf une note déjà écrite,
/// que le livret montre encore et que l'hôte doit pouvoir vider.
fn note_field(view: &View, row: &Row, ctx: &HostContext) -> Component {
    let facility = row.facility;
    let note = facility.note.host_value(ctx);
    let mut field = row
        .field(view, "note", "i18n:host.facility.note", ctx)
        .child(
            TextInput::new()
                .name(row.name("note"))
                .value(note)
                .placeholder("i18n:host.facility.note.placeholder"),
        );
    if note.trim().is_empty() || facility.mode() == MODE_ON_REQUEST {
        field = field.visibleWhen(row.when(MODE_ON_REQUEST));
    }
    field.into()
}

/// Les jours de fermeture, en choix multiple : ils ferment la ligne ces jours-là, quel que soit le
/// mode.
fn closed_days_field(row: &Row) -> Component {
    Field::new()
        .name(row.name("closed_days"))
        .label("i18n:host.facility.closedDays")
        .child(
            ChoiceList::new()
                .name(row.name("closed_days"))
                .multi(true)
                .layout(ChoiceListLayout::Compact)
                .choices(
                    crate::schedule::WEEK
                        .iter()
                        .map(|day| {
                            let key = crate::schedule::day_key(*day);
                            ChoiceOption::new(key, format!("i18n:host.day.{key}"))
                        })
                        .collect(),
                )
                .value(serde_json::to_string(&row.facility.closed_days).unwrap_or_default()),
        )
        .into()
}

/// La saison, du `JJ/MM` au `JJ/MM`.
fn season_fields(view: &View, row: &Row, ctx: &HostContext) -> Vec<Component> {
    let facility = row.facility;
    let date = |key: &str, label: &str, raw: &str, placeholder: &str| -> Component {
        let value = if raw.trim().is_empty() {
            String::new()
        } else {
            season_display(raw)
        };
        row.field(view, key, label, ctx)
            .child(
                TextInput::new()
                    .name(row.name(key))
                    .value(value)
                    .placeholder(placeholder),
            )
            .into()
    };
    vec![
        date(
            "season_from",
            "i18n:host.facility.season.from",
            &facility.season_from,
            "01/06",
        ),
        date(
            "season_to",
            "i18n:host.facility.season.to",
            &facility.season_to,
            "30/09",
        ),
        FieldHint::new()
            .text("i18n:host.facility.season.hint")
            .into(),
    ]
}

/// Les fermetures exceptionnelles : une date, fermé ou une plage, un motif. « Ajouter » rend une
/// rangée de plus ; une rangée laissée vide ne compte pas.
fn exceptions_field(view: &View, row: &Row, ctx: &HostContext) -> Component {
    let facility = row.facility;
    let prefix = row.name("exceptions");
    let count_key = format!("exceptions.{}", facility.id);
    let dated: Vec<_> = facility.dated().cloned().collect();
    let count = ctx
        .input_u64(&count_key)
        .map(|n| n as usize)
        .unwrap_or(0)
        .max(dated.len())
        .min(MAX_EXCEPTIONS);
    let rows: Vec<Component> = (0..count)
        .map(|n| {
            let entry = dated.get(n).cloned().unwrap_or_default();
            let key = |sub: &str| format!("exceptions.{n}.{sub}");
            Stack::new()
                .gap(8.0)
                .children(vec![
                    row.field(view, &key("date"), "i18n:host.exception.date", ctx)
                        .child(
                            DatePicker::new()
                                .name(row.name(&key("date")))
                                .value(entry.date),
                        )
                        .into(),
                    row.field(view, &key("closed"), "i18n:host.exception.closed", ctx)
                        .child(
                            Toggle::new()
                                .name(row.name(&key("closed")))
                                .checked(entry.closed),
                        )
                        .into(),
                    row.field(view, &key("range"), "i18n:host.exception.range", ctx)
                        .visibleWhen(format!("{prefix}.{n}.closed=false"))
                        .child(
                            TimeRange::new()
                                .name(row.name(&key("range")))
                                .value(entry.range),
                        )
                        .into(),
                    row.field(view, &key("reason"), "i18n:host.exception.reason", ctx)
                        .child(
                            TextInput::new()
                                .name(row.name(&key("reason")))
                                .value(entry.reason.host_value(ctx))
                                .placeholder("i18n:host.exception.reason.placeholder"),
                        )
                        .into(),
                ])
                .into()
        })
        .collect();
    let mut list = StepList::new()
        .itemKeyPrefix(prefix.clone())
        .hint("i18n:host.exception.hint")
        .addLabel("i18n:host.exception.add")
        .children(rows);
    let mut children: Vec<Component> = Vec::new();
    if count < MAX_EXCEPTIONS {
        let mut input = Map::new();
        input.insert(count_key, Value::from(count + 1));
        list = list.addAction(Action::emit(
            contracts::shell::SURFACE_INPUT,
            Some(Value::Object(input)),
        ));
    }
    children.push(list.into());
    if count >= MAX_EXCEPTIONS {
        children.push(
            Text::new()
                .text("i18n:host.exception.tooMany")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    Field::new()
        .name(prefix)
        .label("i18n:host.exception.title")
        .children(children)
        .into()
}

/// Les anciens champs en texte libre, tant qu'ils portent une valeur : le livret les lit encore,
/// l'hôte doit pouvoir les vider. Un avertissement dit pourquoi ils ne calculent rien.
fn legacy_fields(view: &View, row: &Row, ctx: &HostContext) -> Vec<Component> {
    let facility = row.facility;
    let mut children: Vec<Component> = Vec::new();
    if let Some(warning) = row
        .stored
        .and_then(|stored| view.message(&format!("facilities.{stored}.hours"), ctx))
    {
        children.push(
            InlineNotice::new()
                .tone(Tone::Warning)
                .message(warning)
                .into(),
        );
    }
    let hours = facility.hours.as_deref().unwrap_or("");
    if !hours.trim().is_empty() {
        children.push(
            Field::new()
                .name(row.name("hours"))
                .label("i18n:host.facility.hours")
                .child(TextInput::new().name(row.name("hours")).value(hours))
                .into(),
        );
    }
    let lines = facility.lines.host_value(ctx);
    if !lines.trim().is_empty() {
        children.push(
            Field::new()
                .name(row.name("lines"))
                .label("i18n:host.facility.lines")
                .child(TextArea::new().name(row.name("lines")).value(lines))
                .into(),
        );
    }
    children
}
