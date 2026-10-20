//! Host configuration, held by the platform (`#[portaki_sdk::config]`).

use portaki_sdk::bundle_text;
use std::collections::BTreeSet;

use crate::schedule::{parse_hm, week_index, DayHours, DaySpan, Schedule};
use chrono::NaiveDate;
use portaki_sdk::config::check;
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::sdui::hours::{overlaps, parse_range, parse_week, Range};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The keys are the names of the host form fields: the platform takes `updateConfig` itself.
#[portaki_sdk::config(legacy = legacy)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModuleConfig {
    /// Combien de lignes la carte d'accueil montre avant « Voir tous les horaires ». `0` : rien
    /// choisi. Flottant : le `NumberInput` envoie un nombre, pas un entier.
    #[field(label = "host.cardLimit.label")]
    pub card_limit: f64,
    /// `local` : des horaires se vérifient après une copie depuis un autre logement (règles
    /// communes §5). Les fermetures exceptionnelles en sont une sous-clé : aucun attribut ne
    /// peut les exclure de la copie.
    #[field(local, label = "config.facilities")]
    pub facilities: Vec<FacilityRow>,
    #[field(label = "host.note.label")]
    pub general_note: I18nText,
}

/// The old KV blob: the facilities as a JSON string, `facilities_json`, before the form slots;
/// each row's `lines` as a list of per-language texts; a row named `name` by an early form.
fn legacy(mut old: Value) -> Value {
    if let Some(object) = old.as_object_mut() {
        map_legacy(object);
    }
    old
}

fn map_legacy(old: &mut Map<String, Value>) {
    let listed = old
        .remove("facilities_json")
        .and_then(|raw| serde_json::from_str::<Value>(raw.as_str()?).ok());
    let held = old
        .get("facilities")
        .and_then(Value::as_array)
        .is_some_and(|rows| !rows.is_empty());
    if let (Some(facilities), false) = (listed, held) {
        old.insert("facilities".into(), facilities);
    }
    for row in old
        .get_mut("facilities")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object_mut)
    {
        if let Some(name) = row.remove("name") {
            row.entry("title").or_insert(name);
        }
        if let Some(Value::Array(lines)) = row.get("lines") {
            let lines = joined_lines(lines);
            row.insert("lines".into(), lines);
        }
    }
}

/// A list of per-language lines as one text per language, one line each — a line missing in a
/// language falls back as it did when shown alone.
fn joined_lines(lines: &[Value]) -> Value {
    let lines: Vec<I18nText> = lines
        .iter()
        .filter_map(|line| serde_json::from_value(line.clone()).ok())
        .collect();
    let languages: BTreeSet<&str> = lines
        .iter()
        .flat_map(|line| {
            ["fr", "en"]
                .into_iter()
                .chain(line.others.keys().map(String::as_str))
        })
        .collect();
    let by_language: Map<String, Value> = languages
        .into_iter()
        .map(|language| {
            let text = lines
                .iter()
                .map(|line| line.get(language).trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            (language.to_string(), Value::String(text))
        })
        .collect();
    Value::Object(by_language)
}

/// Lignes sur la carte : le défaut, et les bornes (spec Horaires §2.1).
pub const DEFAULT_CARD_LIMIT: usize = 3;
pub const MIN_CARD_LIMIT: usize = 1;
pub const MAX_CARD_LIMIT: usize = 6;

/// Combien d'équipements la configuration accepte (§2.2).
pub const MAX_FACILITIES: usize = 30;
/// Combien de fermetures exceptionnelles par équipement (§2.2).
pub const MAX_EXCEPTIONS: usize = 20;
/// Combien de plages par jour (§2.2) : `range_1` … `range_3`, et `maxRanges` de la semaine.
pub const MAX_RANGES: usize = 3;

/// Longueurs des textes d'une ligne (§2.2).
const TITLE_MAX: usize = 40;
const NOTE_MAX: usize = 120;
const REASON_MAX: usize = 80;

/// Les groupes, dans l'ordre où le détail les range (§2.2).
pub const GROUPS: [&str; 3] = ["stay", "equipment", "services"];
/// Le groupe d'une ligne qui n'en dit rien.
pub const DEFAULT_GROUP: &str = "equipment";

/// Comment une ligne donne ses horaires (§2.2).
pub const MODES: [&str; 4] = [MODE_SAME, MODE_BY_DAY, MODE_ALWAYS, MODE_ON_REQUEST];
pub const MODE_SAME: &str = "same";
/// « Selon le jour » : la semaine de `week`, un jour sans plage est fermé.
pub const MODE_BY_DAY: &str = "by_day";
pub const MODE_ALWAYS: &str = "always";
pub const MODE_ON_REQUEST: &str = "on_request";

impl ModuleConfig {
    /// Les lignes de la carte d'accueil, bornées ; le défaut quand rien n'est choisi.
    pub fn card_limit(&self) -> usize {
        if !self.card_limit.is_finite() || self.card_limit <= 0.0 {
            return DEFAULT_CARD_LIMIT;
        }
        (self.card_limit.round() as usize).clamp(MIN_CARD_LIMIT, MAX_CARD_LIMIT)
    }

    /// Ce qui bloque la publication, champ par champ (`card_limit`, `facilities.<i>.<clé>`) — sous
    /// le champ dans le formulaire, et dans `publishReadiness`, pour que les deux disent la même
    /// chose.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        let mut problems = Vec::new();
        if self.card_limit != 0.0 {
            if let Some(error) = check::between(
                self.card_limit,
                MIN_CARD_LIMIT as f64,
                MAX_CARD_LIMIT as f64,
            ) {
                problems.push(("card_limit".to_string(), error));
            }
        }
        let filled = self.facilities.iter().filter(|row| !row.is_blank()).count();
        if filled > MAX_FACILITIES {
            problems.push((
                "facilities".to_string(),
                bundle_text!("host.facilities.tooMany"),
            ));
        }
        for (index, row) in self.facilities.iter().enumerate() {
            for (key, error) in row.problems() {
                problems.push((format!("facilities.{index}.{key}"), error));
            }
        }
        problems
    }

    /// Ce qui se signale sans bloquer : une fermeture exceptionnelle passée, des horaires restés en
    /// texte libre. `today` : le jour au fuseau du logement, `None` sans horloge.
    pub fn warnings(&self, today: Option<NaiveDate>) -> Vec<(String, I18nText)> {
        self.facilities
            .iter()
            .enumerate()
            .flat_map(|(index, row)| {
                row.warnings(today)
                    .into_iter()
                    .map(move |(key, text)| (format!("facilities.{index}.{key}"), text))
            })
            .collect()
    }

    /// Le message à afficher sous `field`, erreur d'abord, avertissement ensuite.
    pub fn message_of(&self, field: &str, today: Option<NaiveDate>) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .chain(self.warnings(today))
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
    }

    pub fn is_empty(&self) -> bool {
        self.parse_facilities().is_empty() && self.general_note.is_blank()
    }

    /// The named rows, for the guest: the form sends its slots, blank ones included.
    pub fn parse_facilities(&self) -> Vec<FacilityRow> {
        self.facilities
            .iter()
            .filter(|f| !f.title.is_blank())
            .cloned()
            .collect()
    }
}

/// A facility, as the form sends it (and its `id`); the platform keeps the other languages.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct FacilityRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub title: I18nText,
    /// One line of text per line. Ancien champ : le panneau ne le montre que tant qu'il porte un
    /// texte, pour que l'hôte puisse le vider.
    pub lines: I18nText,
    /// La phrase que l'hôte a écrite avant les plages. Conservée et affichée telle quelle (le
    /// formulaire la montre tant qu'elle n'est pas vide, avec un avertissement) : rien ici ne la
    /// réécrit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<String>,
    /// « Même horaire tous les jours » : jusqu'à trois plages `HH:MM-HH:MM`, la valeur d'un
    /// `TimeRange` (§2.2). Trois champs plutôt qu'une liste : le formulaire ne sait pas envoyer une
    /// liste de textes sous une ligne (`facilities.0.ranges.0` n'est pas une clé de liste pour lui).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub range_1: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub range_2: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub range_3: String,
    /// « Selon le jour » : la valeur d'un `WeeklyHours`, `mon=09:00-12:00,14:00-19:00;tue=…`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub week: String,
    /// L'ouverture et la fermeture d'avant les plages, en `HH:MM`. Lues tant que les trois plages
    /// sont vides ([`FacilityRow::same_ranges`]) ; le formulaire ne les envoie plus.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub opens_at: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub closes_at: String,
    /// L'ancienne coupure du midi : la deuxième plage, à la lecture.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub break_from: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub break_to: String,
    /// L'ancienne case « 24 h/24 », encore lue : [`FacilityRow::mode`] la remplace.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub all_day: bool,
    /// `same`, `by_day`, `always` ou `on_request` ([`MODES`]). Vide sur une ligne d'avant ce
    /// choix : voir [`FacilityRow::mode`].
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mode: String,
    /// Les fermetures exceptionnelles, datées (§2.2). Y traînent aussi les lignes d'avant `week`
    /// (une par jour de semaine, `id` = `mon` …) et d'anciennes dérogations hebdomadaires (`day`) :
    /// lues, jamais montrées, et remplacées au premier enregistrement d'une fermeture.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<DayHours>,
    /// Les jours de fermeture, `mon` … `sun` (spec Horaires §2.2) : un choix multiple du
    /// formulaire, qui ferme ces jours-là quel que soit le mode.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "days_list"
    )]
    pub closed_days: Vec<String>,
    /// « Comment réserver » : la légende de la ligne, obligatoire « sur demande ».
    pub note: I18nText,
    /// Le groupe sous lequel la ligne se range dans la sous-page : `stay`, `equipment` ou
    /// `services` ([`GROUPS`]). Une ligne d'avant la liste fermée porte encore le texte libre que
    /// l'hôte avait écrit — [`FacilityRow::group_key`] le range, le livret le montre tel quel
    /// jusqu'au prochain enregistrement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Le début de la saison : `JJ/MM` depuis le formulaire, `MM-JJ` sur une ligne d'avant
    /// ([`crate::schedule::parse_month_day`] lit les deux).
    ///
    /// Sans année : une piscine ouvre « de juin à septembre » chaque année. Vides, la ligne est de
    /// toute saison.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub season_from: String,
    /// La fin de la saison.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub season_to: String,
    /// L'icône de la ligne, prise dans le vocabulaire du livret ([`ICONS`]). `None` : jamais
    /// choisie, le formulaire propose celle du nom ([`suggest_icon`]) ; `""` : aucune.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// Une liste de jours, en tableau ou en texte : un dashboard d'avant le choix multiple envoie
/// la valeur cochée seule (`"mon"`), ou du JSON en chaîne (`"[\"mon\"]"`). Refuser la chaîne
/// ferait refuser toute la configuration, et le module rendrait son état d'erreur partout.
fn days_list<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Value::String(text) => serde_json::from_str::<Vec<String>>(&text).unwrap_or_else(|_| {
            text.split(',')
                .map(|day| day.trim().to_string())
                .filter(|day| !day.is_empty())
                .collect()
        }),
        _ => Vec::new(),
    })
}

/// Les icônes proposées à l'hôte, celles du dessin (§2.6) plus les lieux qui reviennent.
///
/// Une liste figée, pas du texte libre : un nom hors du vocabulaire du livret ne dessine rien.
pub const ICONS: [&str; 12] = [
    "key", "logout", "sun", "zap", "sparkles", "building", "package", "recycle", "clock", "wifi",
    "car", "droplet",
];

/// L'icône que le nom appelle (§4 « Icône depuis le nom »), ou `None`.
///
/// ponytail: une table de mots, en français et en anglais ; un nom dans une autre langue n'a pas
/// de proposition, et l'hôte choisit lui-même.
pub fn suggest_icon(name: &str) -> Option<&'static str> {
    const WORDS: &[(&str, &[&str])] = &[
        (
            "key",
            &[
                "arrivée", "arrivee", "check-in", "checkin", "clé", "cle", "key",
            ],
        ),
        ("logout", &["départ", "depart", "check-out", "checkout"]),
        (
            "sparkles",
            &[
                "spa", "sauna", "hammam", "ménage", "menage", "cleaning", "jacuzzi",
            ],
        ),
        (
            "droplet",
            &["piscine", "pool", "laverie", "laundry", "lavage", "douche"],
        ),
        ("sun", &["plage", "beach", "terrasse", "jardin", "garden"]),
        (
            "building",
            &[
                "accueil",
                "réception",
                "reception",
                "conciergerie",
                "bureau",
            ],
        ),
        ("package", &["colis", "parcel", "courrier", "mail"]),
        (
            "recycle",
            &["tri", "déchet", "dechet", "poubelle", "recycl", "waste"],
        ),
        ("car", &["parking", "garage", "voiture"]),
        (
            "zap",
            &[
                "borne",
                "recharge",
                "charging",
                "salle de sport",
                "gym",
                "fitness",
            ],
        ),
        ("wifi", &["wifi", "wi-fi", "internet"]),
    ];
    let name = name.to_lowercase();
    WORDS
        .iter()
        .find(|(_, words)| words.iter().any(|word| name.contains(word)))
        .map(|(icon, _)| *icon)
}

impl FacilityRow {
    /// Le groupe, débarrassé des espaces, ou `None` quand l'hôte n'en a pas donné.
    pub fn group_label(&self) -> Option<&str> {
        self.group
            .as_deref()
            .map(str::trim)
            .filter(|group| !group.is_empty())
    }

    /// Le groupe dans la liste fermée. Un ancien texte libre s'y range par ce qu'il dit
    /// (« Séjour », « Services ») ; le reste, et le vide, vont aux équipements.
    pub fn group_key(&self) -> &'static str {
        let Some(group) = self.group_label() else {
            return DEFAULT_GROUP;
        };
        if let Some(key) = GROUPS.iter().find(|key| **key == group) {
            return key;
        }
        let lower = group.to_lowercase();
        if ["séjour", "sejour", "stay"]
            .iter()
            .any(|word| lower.contains(word))
        {
            "stay"
        } else if lower.contains("service") {
            "services"
        } else {
            DEFAULT_GROUP
        }
    }

    /// Comment la ligne donne ses horaires. Sans choix enregistré : l'ancienne case « 24 h/24 »,
    /// sinon le même horaire tous les jours.
    pub fn mode(&self) -> &'static str {
        if let Some(mode) = MODES.iter().find(|mode| **mode == self.mode.trim()) {
            return mode;
        }
        if self.all_day {
            MODE_ALWAYS
        } else {
            MODE_SAME
        }
    }

    /// Les trois champs de plage, dans l'ordre.
    pub fn range_fields(&self) -> [(&'static str, &str); MAX_RANGES] {
        [
            ("range_1", self.range_1.as_str()),
            ("range_2", self.range_2.as_str()),
            ("range_3", self.range_3.as_str()),
        ]
    }

    /// Les plages de « même horaire » : celles du formulaire, sinon l'ancienne ouverture, sa
    /// fermeture et sa coupure du midi — la migration se fait à la lecture, et le panneau les
    /// montre ainsi dans ses plages.
    pub fn same_ranges(&self) -> Vec<Range> {
        let fields = self.range_fields();
        if fields.iter().any(|(_, raw)| !raw.trim().is_empty()) {
            return fields
                .iter()
                .filter_map(|(_, raw)| parse_range(raw))
                .collect();
        }
        let (Some(opens), Some(closes)) = (parse_hm(&self.opens_at), parse_hm(&self.closes_at))
        else {
            return Vec::new();
        };
        let span = DaySpan { opens, closes };
        match parse_hm(&self.break_from).zip(parse_hm(&self.break_to)) {
            Some((from, to)) if span.holds_break(from, to) => {
                vec![(hm(opens), hm(from)), (hm(to), hm(closes))]
            }
            _ if opens != closes => vec![(hm(opens), hm(closes))],
            _ => Vec::new(),
        }
    }

    /// « Selon le jour » : la semaine du formulaire, sinon les anciennes lignes par jour (une
    /// plage chacune).
    pub fn week_ranges(&self) -> [Vec<Range>; 7] {
        if !self.week.trim().is_empty() {
            return parse_week(&self.week);
        }
        let mut week: [Vec<Range>; 7] = Default::default();
        for (day, entry) in self.by_day_entries() {
            let raw = format!(
                "{}-{}",
                entry.opens_at.as_deref().unwrap_or(""),
                entry.closes_at.as_deref().unwrap_or("")
            );
            if let Some(range) = parse_range(&raw) {
                week[day] = vec![range];
            }
        }
        week
    }

    /// Les fermetures exceptionnelles, telles que le panneau les montre et les numérote : tout
    /// sauf les anciennes lignes de jour de semaine.
    pub fn dated(&self) -> impl Iterator<Item = &DayHours> {
        self.exceptions
            .iter()
            .filter(|entry| entry.weekday().is_none())
    }

    /// La fermeture exceptionnelle de `date`, s'il y en a une.
    pub fn exception_on(&self, date: NaiveDate) -> Option<&DayHours> {
        self.dated().find(|entry| entry.date() == Some(date))
    }

    /// Ce qui ne va pas sur cette ligne, par clé de champ. Une ligne vide n'a rien à dire : un
    /// emplacement laissé n'est pas une erreur.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        if self.is_blank() {
            return Vec::new();
        }
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        if self.title.is_blank() {
            problems.push((
                "title".to_string(),
                bundle_text!("host.facility.name.required"),
            ));
        } else if let Some(error) = self
            .title
            .by_language()
            .find_map(|(_, title)| check::max_chars(title, TITLE_MAX))
        {
            problems.push(("title".to_string(), error));
        }
        if self.mode() == MODE_ON_REQUEST && self.note.is_blank() {
            problems.push((
                "note".to_string(),
                bundle_text!("host.facility.note.required"),
            ));
        } else if let Some(error) = self
            .note
            .by_language()
            .find_map(|(_, note)| check::max_chars(note, NOTE_MAX))
        {
            problems.push(("note".to_string(), error));
        }
        if self.mode() == MODE_SAME {
            let mut valid = Vec::new();
            let mut last = None;
            for (key, raw) in self.range_fields() {
                if raw.trim().is_empty() {
                    continue;
                }
                match parse_range(raw) {
                    Some(range) => {
                        valid.push(range);
                        last = Some(key);
                    }
                    None => problems.extend(range_error(raw).map(|e| (key.to_string(), e))),
                }
            }
            if let (true, Some(key)) = (overlaps(&valid), last) {
                problems.push((
                    key.to_string(),
                    bundle_text!("host.facility.ranges.overlap"),
                ));
            }
        }
        if self.mode() == MODE_BY_DAY && !self.week.trim().is_empty() {
            let week = parse_week(&self.week);
            if let Some(day) = (0..7).find(|&day| overlaps(&week[day])) {
                problems.push(("week".to_string(), overlap_on(crate::schedule::WEEK[day])));
            }
        }
        if self.season_from.trim().is_empty() != self.season_to.trim().is_empty() {
            let missing = if self.season_from.trim().is_empty() {
                "season_from"
            } else {
                "season_to"
            };
            problems.push((
                missing.to_string(),
                bundle_text!("host.facility.season.both"),
            ));
        }
        for (index, entry) in self.dated().enumerate() {
            if !entry.closed
                && !entry.range.trim().is_empty()
                && parse_range(&entry.range).is_none()
            {
                problems.extend(
                    range_error(&entry.range).map(|e| (format!("exceptions.{index}.range"), e)),
                );
            }
            if let Some(error) = entry
                .reason
                .by_language()
                .find_map(|(_, reason)| check::max_chars(reason, REASON_MAX))
            {
                problems.push((format!("exceptions.{index}.reason"), error));
            }
        }
        problems
    }

    /// Ce qui se signale sans bloquer.
    ///
    /// Une fermeture passée : « Cette date est passée. » sous sa date, mais la publication ne s'y
    /// arrête pas — une date juste écoulée n'est pas une faute de l'hôte, et le livret l'ignore
    /// déjà. Des horaires en texte libre, sans plage : le livret les affiche, sans pouvoir dire si
    /// c'est ouvert.
    pub fn warnings(&self, today: Option<NaiveDate>) -> Vec<(String, I18nText)> {
        if self.is_blank() {
            return Vec::new();
        }
        let mut warnings: Vec<(String, I18nText)> = self
            .dated()
            .enumerate()
            .filter(|(_, entry)| matches!((today, entry.date()), (Some(today), Some(date)) if date < today))
            .map(|(index, _)| {
                (
                    format!("exceptions.{index}.date"),
                    bundle_text!("host.exception.past"),
                )
            })
            .collect();
        let prose =
            self.hours.as_deref().is_some_and(|h| !h.trim().is_empty()) || !self.lines.is_blank();
        if self.mode() == MODE_SAME && prose && self.same_ranges().is_empty() {
            warnings.push((
                "hours".to_string(),
                bundle_text!("host.facility.hours.legacy"),
            ));
        }
        warnings
    }

    /// L'icône, et seulement si elle est du vocabulaire : le reste ne dessinerait rien.
    pub fn icon_name(&self) -> Option<&str> {
        self.icon
            .as_deref()
            .map(str::trim)
            .filter(|icon| ICONS.contains(icon))
    }
}

/// Le message d'une plage illisible : l'heure mal écrite, ou une fin égale au début.
fn range_error(raw: &str) -> Option<I18nText> {
    let (start, end) = raw.split_once('-').unwrap_or((raw, ""));
    let (start, end) = (start.trim(), end.trim());
    check::time(start)
        .or_else(|| check::time(end))
        .or_else(|| check::time_range(start, end))
}

/// `HH:MM` depuis des minutes après minuit.
fn hm(minutes: u32) -> String {
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

impl FacilityRow {
    /// Les horaires de cette ligne, tels que le calcul les lit.
    pub fn schedule(&self) -> Schedule {
        let mode = self.mode();
        let spans = |ranges: &[Range]| -> Vec<DaySpan> {
            ranges
                .iter()
                .filter_map(|(start, end)| {
                    Some(DaySpan {
                        opens: parse_hm(start)?,
                        closes: parse_hm(end)?,
                    })
                })
                .collect()
        };
        let mut week: [Vec<DaySpan>; 7] = Default::default();
        if mode == MODE_SAME {
            let usual = spans(&self.same_ranges());
            if !usual.is_empty() {
                week = std::array::from_fn(|_| usual.clone());
                // Les anciennes dérogations d'un jour de semaine (`day`), lues comme avant.
                for entry in self.exceptions.iter().filter(|e| !e.day.trim().is_empty()) {
                    let Some(day) = week_index(&entry.day) else {
                        continue;
                    };
                    let own = entry
                        .opens_at
                        .as_deref()
                        .and_then(parse_hm)
                        .zip(entry.closes_at.as_deref().and_then(parse_hm));
                    if entry.closed {
                        week[day].clear();
                    } else if let Some((opens, closes)) = own {
                        week[day] = vec![DaySpan { opens, closes }];
                    }
                }
            }
        } else if mode == MODE_BY_DAY {
            let ranges = self.week_ranges();
            week = std::array::from_fn(|day| spans(&ranges[day]));
        }
        // Calculable avant les jours de fermeture : une ligne fermée tous les jours dit « Fermé »,
        // pas sa phrase.
        let structured = mode == MODE_ALWAYS || week.iter().any(|day| !day.is_empty());
        let mut closed = [false; 7];
        for day in self.closed_days.iter().filter_map(|day| week_index(day)) {
            closed[day] = true;
            week[day].clear();
        }
        Schedule {
            all_day: mode == MODE_ALWAYS,
            on_request: mode == MODE_ON_REQUEST,
            structured,
            week,
            closed,
            dated: self
                .dated()
                .filter_map(|entry| Some((entry.date()?, entry.spans())))
                .collect(),
            // Les deux bornes, ou aucune : une saison à une seule date ne dit pas quand elle
            // s'arrête, et la deviner fermerait la ligne à une date inventée.
            season: crate::schedule::parse_month_day(&self.season_from)
                .zip(crate::schedule::parse_month_day(&self.season_to)),
        }
    }

    /// Les anciennes lignes « selon le jour », chacune avec l'index de son jour dans [`WEEK`]
    /// (lundi = 0) : le jour vient de `id` (le champ caché de l'ancien formulaire), sinon de
    /// `day`.
    ///
    /// [`WEEK`]: crate::schedule::WEEK
    pub fn by_day_entries(&self) -> Vec<(usize, &DayHours)> {
        self.exceptions
            .iter()
            .filter_map(|entry| Some((entry.weekday()?, entry)))
            .collect()
    }
}

impl FacilityRow {
    /// Nothing the form shows: a slot the host left (or emptied).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank()
            && self.hours.as_deref().is_none_or(|h| h.trim().is_empty())
            && self.lines.is_blank()
            && self.note.is_blank()
            && self
                .range_fields()
                .iter()
                .all(|(_, raw)| raw.trim().is_empty())
            && self.week.trim().is_empty()
    }

    /// The non-blank lines in `locale`.
    pub fn lines(&self, locale: &str) -> Vec<String> {
        self.lines
            .get(locale)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(String::from)
            .collect()
    }
}

/// « … le samedi » : le jour dans la langue de chaque texte, en minuscule en français (« le
/// samedi »), tel quel ailleurs (« on Saturday »).
fn overlap_on(day: chrono::Weekday) -> I18nText {
    let mut text = bundle_text!("host.facility.ranges.overlapOn");
    let fill = |lang: &str, template: &mut String| {
        let name = portaki_sdk::host::time::weekday_name(day, lang);
        let name = if lang == "fr" {
            name.to_lowercase()
        } else {
            name.to_string()
        };
        *template = template.replace("{day}", &name);
    };
    fill("fr", &mut text.fr);
    fill("en", &mut text.en);
    for (lang, template) in &mut text.others {
        fill(lang, template);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Weekday;
    use portaki_test_utils::MockContext;
    use serde_json::json;

    #[test]
    fn a_form_row_reads_with_blank_slots_left_out() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "facilities": [{ "title": "Piscine", "hours": "9 h – 20 h" }, { "title": "", "hours": "" }],
            "general_note": "Horaires indicatifs"
        }))
        .unwrap();
        let facilities = config.parse_facilities();
        assert_eq!(facilities.len(), 1);
        assert_eq!(facilities[0].title.get("en"), "Piscine");
        assert_eq!(config.general_note.get("en"), "Horaires indicatifs");
    }

    #[test]
    fn legacy_facilities_json_becomes_facilities() {
        let mapped = legacy(json!({
            "facilities_json": r#"[{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"hours":"08:00 – 20:00","note":{"fr":"Bonnet"}}]"#,
            "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
        }));
        assert_eq!(
            mapped,
            json!({
                "facilities": [{ "id": "pool", "title": { "fr": "Piscine", "en": "Pool" },
                                 "hours": "08:00 – 20:00", "note": { "fr": "Bonnet" } }],
                "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.facilities[0].title.get("en"), "Pool");
        assert_eq!(config.facilities[0].note.get("en"), "Bonnet");
        assert_eq!(config.general_note.get("en"), "Indicative hours");
        // A list already there wins over the old string; an unreadable string imports nothing.
        let kept = legacy(json!({ "facilities": [{ "title": "Spa" }], "facilities_json": "[]" }));
        assert_eq!(kept, json!({ "facilities": [{ "title": "Spa" }] }));
        assert_eq!(legacy(json!({ "facilities_json": "[oops" })), json!({}));
        // An empty list (the old reader's « nothing saved yet ») still takes the string.
        let empty = legacy(json!({ "facilities": [], "facilities_json": r#"[{"title":"Spa"}]"# }));
        assert_eq!(empty, json!({ "facilities": [{ "title": "Spa" }] }));
    }

    #[test]
    fn legacy_lines_become_one_text_per_language() {
        let mapped = legacy(json!({ "facilities": [{
            "id": "pool",
            "title": { "fr": "Piscine", "en": "Pool", "de": "Schwimmbad" },
            "lines": [
                { "fr": "Tous les jours", "en": "Every day" },
                { "fr": "  " , "en": "" },
                { "fr": "Enfants accompagnés", "de": "Kinder begleitet" }
            ]
        }] }));
        assert_eq!(
            mapped["facilities"][0]["lines"],
            json!({
                "fr": "Tous les jours\nEnfants accompagnés",
                "en": "Every day\nEnfants accompagnés",
                "de": "Tous les jours\nKinder begleitet"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(
            config.facilities[0].lines("en-US"),
            ["Every day", "Enfants accompagnés"]
        );
        assert_eq!(config.facilities[0].title.get("de"), "Schwimmbad");
    }

    #[test]
    fn legacy_name_becomes_title_and_a_plain_note_stays() {
        let mapped = legacy(json!({
            "facilities": [{ "name": "Accueil", "hours": "16:00" }, { "name": "x", "title": { "fr": "Spa" } }],
            "general_note": "Horaires indicatifs"
        }));
        assert_eq!(
            mapped,
            json!({
                "facilities": [{ "title": "Accueil", "hours": "16:00" }, { "title": { "fr": "Spa" } }],
                "general_note": "Horaires indicatifs"
            })
        );
        let config: ModuleConfig = serde_json::from_value(mapped).unwrap();
        assert_eq!(config.facilities[0].title.get("fr"), "Accueil");
        assert_eq!(config.general_note.get("en"), "Horaires indicatifs");
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        let old = json!({
            "facilities_json": r#"[{"id":"pool","title":{"fr":"Piscine","en":"Pool"},"lines":[{"fr":"Tous les jours","en":"Every day"}],"hours":"08:00 – 20:00"}]"#,
            "general_note": { "fr": "Horaires indicatifs", "en": "Indicative hours" }
        });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .run(|ctx| {
                let config = ModuleConfig::load(&ctx).unwrap();
                let facilities = config.parse_facilities();
                assert_eq!(facilities[0].title.get("en"), "Pool");
                assert_eq!(facilities[0].lines("en"), ["Every day"]);
                assert_eq!(config.general_note.get("en"), "Indicative hours");
            });
        MockContext::guest()
            .with_kv("config", serde_json::to_vec(&old).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert_eq!(ModuleConfig::load(&ctx).unwrap(), ModuleConfig::default()));
    }

    /// Sans choix enregistré, l'ancienne case « 24 h/24 » décide ; un choix l'emporte sur elle.
    #[test]
    fn the_mode_reads_the_old_all_day_box() {
        let row = |value| serde_json::from_value::<FacilityRow>(value).unwrap();
        assert_eq!(row(json!({ "title": "Spa" })).mode(), MODE_SAME);
        assert_eq!(row(json!({ "all_day": true })).mode(), MODE_ALWAYS);
        assert_eq!(
            row(json!({ "all_day": true, "mode": "on_request" })).mode(),
            MODE_ON_REQUEST
        );
        assert!(
            !row(json!({ "all_day": true, "mode": "same" }))
                .schedule()
                .all_day
        );
    }

    /// Un ancien groupe en texte libre se range par ce qu'il dit.
    #[test]
    fn an_old_free_text_group_finds_its_key() {
        let key = |group: &str| {
            FacilityRow {
                group: Some(group.to_string()),
                ..FacilityRow::default()
            }
            .group_key()
        };
        assert_eq!(key("Séjour"), "stay");
        assert_eq!(key("Services à la carte"), "services");
        assert_eq!(key("Piscine & spa"), DEFAULT_GROUP);
        assert_eq!(key("  "), DEFAULT_GROUP);
        assert_eq!(key("services"), "services");
    }

    /// Les erreurs nomment leur champ, ligne comprise ; une ligne vide n'en a pas.
    #[test]
    fn problems_name_their_field() {
        let config: ModuleConfig = serde_json::from_value(json!({
            "card_limit": 9,
            "facilities": [
                { "title": "", "hours": "" },
                { "title": "", "note": "Réception" },
                { "title": "Spa", "mode": "on_request" },
                { "title": "Piscine", "range_1": "25:00-26:00", "season_from": "01/06" }
            ]
        }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            [
                "card_limit",
                "facilities.1.title",
                "facilities.2.note",
                "facilities.3.range_1",
                "facilities.3.season_to"
            ]
        );
        let fr = |field: &str| {
            config
                .message_of(field, None)
                .unwrap()
                .get("fr")
                .to_string()
        };
        assert_eq!(fr("card_limit"), "Entre 1 et 6.");
        assert_eq!(fr("facilities.1.title"), "Donnez un nom.");
        assert_eq!(fr("facilities.2.note"), "Expliquez comment le demander.");
        assert_eq!(
            fr("facilities.3.season_to"),
            "Indiquez les deux dates de la saison."
        );
        assert_eq!(config.card_limit(), MAX_CARD_LIMIT);
        assert_eq!(ModuleConfig::default().card_limit(), DEFAULT_CARD_LIMIT);
    }

    /// « 30 équipements au maximum. », au-delà de trente lignes remplies.
    #[test]
    fn thirty_facilities_at_most() {
        let rows: Vec<Value> = (0..=MAX_FACILITIES)
            .map(|i| json!({ "title": format!("Équipement {i}") }))
            .collect();
        let config: ModuleConfig = serde_json::from_value(json!({ "facilities": rows })).unwrap();
        assert_eq!(
            config.message_of("facilities", None).unwrap().get("fr"),
            "30 équipements au maximum."
        );
    }

    /// Un nom de 41 caractères, une note de 121, un motif de 81 : chacun sous son champ.
    #[test]
    fn texts_are_bounded() {
        let config: ModuleConfig = serde_json::from_value(json!({ "facilities": [{
            "title": "x".repeat(41),
            "note": "y".repeat(121),
            "exceptions": [{ "date": "2026-12-24", "closed": true, "reason": "z".repeat(81) }]
        }] }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            [
                "facilities.0.title",
                "facilities.0.note",
                "facilities.0.exceptions.0.reason"
            ]
        );
    }

    /// Même horaire : une plage qui finit quand elle commence, et deux plages qui se chevauchent,
    /// sous la plage fautive.
    #[test]
    fn same_hours_flag_an_empty_range_and_an_overlap() {
        let config: ModuleConfig = serde_json::from_value(json!({ "facilities": [
            { "title": "Spa", "range_1": "09:00-09:00" },
            { "title": "Bar", "range_1": "08:30-12:30", "range_2": "12:00-18:00" },
            { "title": "Club", "range_1": "22:00-02:00", "range_2": "23:00-23:30" },
            { "title": "Piscine", "range_1": "08:30-12:00", "range_2": "14:00-18:00",
              "range_3": "20:00-23:00" }
        ] }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            fields,
            [
                "facilities.0.range_1",
                "facilities.1.range_2",
                "facilities.2.range_2"
            ]
        );
        let fr = |field: &str| {
            config
                .message_of(field, None)
                .unwrap()
                .get("fr")
                .to_string()
        };
        assert_eq!(
            fr("facilities.0.range_1"),
            "L'heure de fin doit être différente de l'heure de début."
        );
        assert_eq!(fr("facilities.1.range_2"), "Deux plages se chevauchent.");
        assert_eq!(
            config.facilities[3].schedule().spans_on(Weekday::Mon).len(),
            3
        );
    }

    /// Selon le jour : la semaine du `WeeklyHours`, et le jour du chevauchement nommé dans les deux
    /// langues.
    #[test]
    fn by_day_names_the_day_of_an_overlap() {
        let config: ModuleConfig = serde_json::from_value(json!({ "facilities": [{
            "title": "Spa", "mode": "by_day",
            "week": "mon=09:00-12:00;sat=09:00-13:00,12:00-18:00"
        }] }))
        .unwrap();
        let error = config.message_of("facilities.0.week", None).unwrap();
        assert_eq!(error.get("fr"), "Deux plages se chevauchent le samedi.");
        assert_eq!(error.get("en"), "Two time ranges overlap on Saturday.");
        let schedule = config.facilities[0].schedule();
        assert_eq!(schedule.spans_on(Weekday::Mon).len(), 1);
        assert!(schedule.spans_on(Weekday::Tue).is_empty());
    }

    /// Les anciennes heures deviennent des plages à la lecture : ouverture, fermeture et coupure
    /// font deux plages ; une coupure qui ne tient pas dans la journée est ignorée.
    #[test]
    fn old_hours_read_as_ranges() {
        let row = |value| serde_json::from_value::<FacilityRow>(value).unwrap();
        let reception = row(
            json!({ "title": "Accueil", "opens_at": "08:30", "closes_at": "18:00",
                                    "break_from": "12:00", "break_to": "14:00" }),
        );
        assert_eq!(
            reception.same_ranges(),
            [
                ("08:30".to_string(), "12:00".to_string()),
                ("14:00".to_string(), "18:00".to_string())
            ]
        );
        for (opens, closes, from, to) in [
            ("08:30", "18:00", "14:00", "12:00"),
            ("08:30", "18:00", "19:00", "20:00"),
            ("22:00", "02:00", "23:00", "23:30"),
        ] {
            let row = row(json!({ "opens_at": opens, "closes_at": closes,
                                  "break_from": from, "break_to": to }));
            assert_eq!(row.same_ranges().len(), 1, "{from} – {to}");
        }
        // Une plage saisie l'emporte sur les anciennes heures.
        let typed =
            row(json!({ "opens_at": "08:00", "closes_at": "20:00", "range_1": "10:00-12:00" }));
        assert_eq!(
            typed.same_ranges(),
            [("10:00".to_string(), "12:00".to_string())]
        );
    }

    /// Les anciennes lignes « selon le jour » (une par jour, `id` = le jour) deviennent la semaine
    /// à la lecture, et ne comptent pas comme des fermetures exceptionnelles.
    #[test]
    fn old_by_day_rows_read_as_a_week() {
        let row: FacilityRow = serde_json::from_value(json!({
            "title": "Spa", "mode": "by_day",
            "exceptions": [
                { "id": "mon", "opens_at": "09:00", "closes_at": "12:00" },
                { "id": "tue" },
                { "id": "wed", "opens_at": "14:00", "closes_at": "18:00" },
                { "date": "2026-12-24", "closed": true }
            ]
        }))
        .unwrap();
        assert_eq!(
            portaki_sdk::sdui::hours::format_week(&row.week_ranges()),
            "mon=09:00-12:00;wed=14:00-18:00"
        );
        assert_eq!(row.dated().count(), 1);
        let schedule = row.schedule();
        // Mercredi 15 juillet 2026, 10 h UTC : la plage du mercredi n'a pas commencé.
        let wednesday = "2026-07-15T10:00:00Z".parse().unwrap();
        assert_eq!(
            schedule.state_at(wednesday, None),
            Some(crate::schedule::State::OpensAt(14 * 60))
        );
    }

    /// Un jour de fermeture ferme la ligne ce jour-là, quel que soit le mode.
    #[test]
    fn a_closed_day_closes_the_row_that_day() {
        let row: FacilityRow = serde_json::from_value(json!({
            "title": "Spa", "mode": "always", "closed_days": ["mon"]
        }))
        .unwrap();
        let schedule = row.schedule();
        assert!(schedule.closed_on(Weekday::Mon));
        assert!(!schedule.closed_on(Weekday::Tue));
        // Lundi 13 juillet 2026, 10 h UTC.
        let monday = "2026-07-13T10:00:00Z".parse().unwrap();
        assert_eq!(
            schedule.state_at(monday, None),
            Some(crate::schedule::State::Closed)
        );
        for other in [
            json!({ "mode": "same", "range_1": "09:00-18:00" }),
            json!({ "mode": "by_day", "week": "mon=09:00-18:00;tue=09:00-18:00" }),
        ] {
            let mut value = json!({ "title": "Spa", "closed_days": ["mon"] });
            value
                .as_object_mut()
                .unwrap()
                .extend(other.as_object().unwrap().clone());
            let row: FacilityRow = serde_json::from_value(value).unwrap();
            assert!(row.schedule().spans_on(Weekday::Mon).is_empty());
            assert_eq!(row.schedule().spans_on(Weekday::Tue).len(), 1);
        }
    }

    /// Une dérogation hebdomadaire sans heures habituelles ne fait pas d'une ligne « même
    /// horaire » un horaire calculé : la phrase de l'hôte reste.
    #[test]
    fn a_named_exception_alone_keeps_the_sentence() {
        let row: FacilityRow = serde_json::from_value(json!({
            "title": "Spa", "mode": "same",
            "exceptions": [{ "day": "wed", "opens_at": "14:00", "closes_at": "18:00" }]
        }))
        .unwrap();
        assert!(!row.schedule().is_structured());
    }

    /// Une fermeture passée : « Cette date est passée. » en avertissement, jamais en erreur ; des
    /// horaires restés en texte libre, de même.
    #[test]
    fn a_past_closure_and_free_text_hours_only_warn() {
        let config: ModuleConfig = serde_json::from_value(json!({ "facilities": [{
            "title": "Piscine", "hours": "9 h – 20 h",
            "exceptions": [
                { "date": "2026-10-01", "closed": true },
                { "date": "2026-12-24", "closed": true }
            ]
        }] }))
        .unwrap();
        assert!(config.problems().is_empty());
        let today = NaiveDate::from_ymd_opt(2026, 10, 12);
        let warnings: Vec<String> = config.warnings(today).into_iter().map(|(f, _)| f).collect();
        assert_eq!(
            warnings,
            ["facilities.0.exceptions.0.date", "facilities.0.hours"]
        );
        assert_eq!(
            config
                .message_of("facilities.0.exceptions.0.date", today)
                .unwrap()
                .get("fr"),
            "Cette date est passée."
        );
    }

    /// L'icône que le nom appelle.
    #[test]
    fn the_name_suggests_an_icon() {
        assert_eq!(suggest_icon("Piscine chauffée"), Some("droplet"));
        assert_eq!(suggest_icon("Réception"), Some("building"));
        assert_eq!(suggest_icon("Check-out"), Some("logout"));
        assert_eq!(suggest_icon("Sauna"), Some("sparkles"));
        assert_eq!(suggest_icon("Bibliothèque"), None);
        assert!(suggest_icon("Parking").is_some_and(|icon| ICONS.contains(&icon)));
    }

    /// Un jour coché arrive en tableau, en JSON dans une chaîne, ou seul : tous se relisent.
    #[test]
    fn closed_days_read_in_every_shape() {
        let days = |value| {
            serde_json::from_value::<FacilityRow>(json!({ "closed_days": value }))
                .unwrap()
                .closed_days
        };
        assert_eq!(days(json!(["mon", "sun"])), ["mon", "sun"]);
        assert_eq!(days(json!("[\"mon\"]")), ["mon"]);
        assert_eq!(days(json!("mon")), ["mon"]);
        assert!(days(json!(null)).is_empty());
    }
}
