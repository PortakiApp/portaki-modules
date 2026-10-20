//! Structured house-rules payload (not TipTap).

use portaki_sdk::bundle_text;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Le poids d'une règle, tel que l'hôte le pose (§2.8).
///
/// Liste fermée, donc indépendante de la langue : elle se recopie dans chaque payload comme
/// l'icône. `Neutral` est l'absence de poids — une règle qu'on énonce sans la souligner.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleStatus {
    /// « Information » : une règle qu'on énonce sans la souligner.
    #[default]
    Neutral,
    Important,
    Allowed,
    /// « Interdit » (spec Règlement §2.2).
    Forbidden,
}

impl RuleStatus {
    /// Lit un statut venu du formulaire hôte ou d'un payload stocké. `ok` est l'orthographe du
    /// mockup pour `allowed` ; tout le reste retombe sur neutre plutôt que d'échouer.
    pub fn from_wire(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "important" => Self::Important,
            "allowed" | "ok" => Self::Allowed,
            "forbidden" => Self::Forbidden,
            _ => Self::Neutral,
        }
    }

    /// Le nom qui part dans le formulaire hôte (valeur du `Select`).
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Important => "important",
            Self::Allowed => "allowed",
            Self::Forbidden => "forbidden",
        }
    }

    /// Le glyphe du tableau hôte et des cartes « Type » : ×, !, ✓, i — ton neutre partout.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Forbidden => "×",
            Self::Important => "!",
            Self::Allowed => "✓",
            Self::Neutral => "i",
        }
    }
}

/// One rule row shown in the guest booklet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuleItem {
    /// Identifiant stable, le même dans chaque langue : c'est lui qui aligne les langues et que
    /// le tableau hôte renvoie. Une règle d'avant les identifiants reçoit `r{rang}` à la lecture
    /// ([`RulesBundle::from_row`]), gardé au premier enregistrement.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// Lucide / design icon name (`clock-circle`, `x`, …).
    #[serde(default)]
    pub icon: String,
    /// Short rule title.
    #[serde(default)]
    pub title: String,
    /// Optional supporting line.
    #[serde(default)]
    pub subtitle: String,
    /// Poids de la règle — partagé entre les langues, comme l'icône.
    #[serde(default)]
    pub status: RuleStatus,
    /// Le thème, dans la liste fermée ([`THEMES`]) : il se recopie entre les langues comme le
    /// statut. Une règle d'avant la liste porte encore le texte libre que l'hôte avait écrit dans
    /// sa langue ; le livret le montre tel quel jusqu'au prochain enregistrement.
    #[serde(default)]
    pub theme: String,
    /// Les heures de la règle, `22:00 – 08:00` — partagées entre les langues.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hours: String,
}

/// Longueurs d'une règle (§2.2).
pub const TITLE_MAX: usize = 60;
pub const DETAIL_MAX: usize = 280;

/// Combien de règles le règlement accepte (§2.2).
pub const MAX_RULES: usize = 30;

/// Les thèmes de la spec (§2.2), dans l'ordre du sélecteur.
pub const THEMES: [&str; 8] = [
    "noise", "pets", "smoking", "parties", "visitors", "cleaning", "safety", "other",
];

/// Les mots qui font proposer un thème (§4), en minuscules, sans accents à respecter : on cherche
/// une sous-chaîne du titre et de la précision. Français et anglais, les deux langues des modèles.
const THEME_WORDS: [(&str, &[&str]); 7] = [
    (
        "noise",
        &[
            "silence", "bruit", "calme", "musique", "noise", "quiet", "music",
        ],
    ),
    (
        "pets",
        &["animal", "animaux", "chien", "chat", "pet", "dog", "cat"],
    ),
    ("smoking", &["fum", "tabac", "cigarette", "smok", "vap"]),
    (
        "parties",
        &[
            "fête",
            "fete",
            "soirée",
            "party",
            "parties",
            "événement",
            "event",
        ],
    ),
    ("visitors", &["visiteur", "invité", "visitor", "guest"]),
    (
        "cleaning",
        &[
            "ménage",
            "poubelle",
            "déchet",
            "vaisselle",
            "clean",
            "trash",
            "rubbish",
            "dishes",
        ],
    ),
    (
        "safety",
        &[
            "sécurité",
            "feu",
            "bougie",
            "piscine",
            "safety",
            "fire",
            "candle",
            "pool",
        ],
    ),
];

/// Le thème que le texte d'une règle suggère, ou `None`.
pub fn suggest_theme(text: &str) -> Option<&'static str> {
    let text = text.to_lowercase();
    THEME_WORDS
        .iter()
        .find(|(_, words)| words.iter().any(|word| text.contains(word)))
        .map(|(key, _)| *key)
}

impl RuleItem {
    /// Le thème proposé depuis le titre et la précision (§4), s'il diffère de « autre ».
    pub fn suggested_theme(&self) -> Option<&'static str> {
        suggest_theme(&format!("{} {}", self.title, self.subtitle))
    }

    /// Les heures pour le livret : `22:00 – 08:00` ; un ancien texte libre reste tel quel.
    pub fn hours_label(&self) -> String {
        let raw = self.hours.trim();
        match portaki_sdk::sdui::hours::parse_range(&range_wire(raw)) {
            Some((start, end)) => format!("{start} – {end}"),
            None => raw.to_string(),
        }
    }

    /// L'icône du livret : celle de la règle, sinon celle de son thème (§9 cas 4).
    pub fn shown_icon(&self) -> &str {
        if !self.icon.trim().is_empty() {
            return self.icon.trim();
        }
        match self.theme_key() {
            Some("noise") => "clock-circle",
            Some("pets") => "gift",
            Some("smoking") => "x",
            Some("parties" | "visitors") => "users",
            _ => "check-circle",
        }
    }

    /// Le thème dans la liste, ou `None` pour un ancien texte libre (ou rien).
    pub fn theme_key(&self) -> Option<&'static str> {
        THEMES
            .iter()
            .find(|key| **key == self.theme.trim())
            .copied()
    }

    /// Ni titre ni précision : une ligne vidée, que l'enregistrement écarte sans rien dire.
    pub fn is_blank(&self) -> bool {
        self.title.trim().is_empty() && self.subtitle.trim().is_empty()
    }

    /// Les erreurs de la spec (§2.2), par champ du formulaire : un titre, s'il y a une précision
    /// (« Écrivez la règle. ») ; titre ≤ 60, précision ≤ 280.
    pub fn problems(&self) -> Vec<(&'static str, portaki_sdk::contracts::i18n::I18nText)> {
        use portaki_sdk::config::check::max_chars;
        let title = if self.title.trim().is_empty() && !self.is_blank() {
            Some(bundle_text!("host.rule.title.required"))
        } else {
            max_chars(&self.title, TITLE_MAX)
        };
        [
            ("title", title),
            ("detail", max_chars(&self.subtitle, DETAIL_MAX)),
        ]
        .into_iter()
        .filter_map(|(field, error)| Some((field, error?)))
        .collect()
    }

    /// Le titre du thème pour le livret : traduit pour une clé, tel quel pour un ancien texte.
    pub fn theme_title(&self) -> String {
        match self.theme_key() {
            Some(key) => format!("i18n:rule.theme.{key}"),
            None => self.theme.trim().to_string(),
        }
    }
}

/// Une plage d'heures au format du `TimeRange` (`22:00-08:00`), depuis ce qui est stocké — les
/// règles d'avant écrivaient `22:00 – 08:00`. Illisible : la chaîne telle quelle.
pub fn range_wire(raw: &str) -> String {
    let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    compact.replace(['–', '—'], "-")
}

/// Locale payload for one language.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RulesPayload {
    #[serde(default)]
    pub items: Vec<RuleItem>,
    /// Les règles sur la carte d'accueil — porté par le bundle, recopié ici pour le livret.
    #[serde(skip)]
    pub card_limit: usize,
}

impl RulesPayload {
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Self::default();
        }
        serde_json::from_str(trimmed).unwrap_or_default()
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|item| item.title.trim().is_empty())
    }

    /// Les règles qui portent un titre — une ligne vide ne s'affiche pas.
    pub fn named(&self) -> impl Iterator<Item = &RuleItem> {
        self.items
            .iter()
            .filter(|item| !item.title.trim().is_empty())
    }

    /// La règle d'identifiant `id`.
    pub fn find(&self, id: &str) -> Option<&RuleItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Donne `r{rang}` aux règles d'avant les identifiants : les langues étaient alignées par rang,
    /// le même rang donne donc le même identifiant d'une langue à l'autre.
    fn backfill_ids(&mut self) {
        for (index, item) in self.items.iter_mut().enumerate() {
            if item.id.trim().is_empty() {
                item.id = format!("r{index}");
            }
        }
    }

    /// Groupées par thème, dans l'ordre où l'hôte a posé les règles — c'est ce que fait le mockup
    /// (`[...new Set(rules.map(r => r.group))]`), et l'ordre d'apparition est une intention.
    ///
    /// Le regroupement se fait sur le thème normalisé (sans casse ni espaces de bord) mais garde la
    /// première orthographe rencontrée : « Piscine » et « piscine » sont le même thème.
    ///
    /// Les règles sans thème forment un bloc sans titre, et ce bloc passe en dernier : un hôte qui
    /// n'a rangé qu'une partie de ses règles garde son groupement, et le reste se lit à la fin
    /// plutôt qu'en tête de page sans qu'on sache ce qu'il annonce.
    pub fn by_theme(&self) -> Vec<(String, Vec<&RuleItem>)> {
        let mut groups: Vec<(String, String, Vec<&RuleItem>)> = Vec::new();
        for item in self.named() {
            let label = item.theme_title();
            let key = label.to_lowercase();
            match groups.iter_mut().find(|(k, _, _)| *k == key) {
                Some((_, _, rules)) => rules.push(item),
                None => groups.push((key, label, vec![item])),
            }
        }
        groups.sort_by_key(|(key, _, _)| key.is_empty());
        groups
            .into_iter()
            .map(|(_, label, rules)| (label, rules))
            .collect()
    }
}

/// N-language storage written into `content_fr` (`content_en` cleared after migrate).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RulesBundle {
    #[serde(default)]
    pub by_lang: BTreeMap<String, RulesPayload>,
    /// « Règles sur la carte » (§2.1). Absent : [`DEFAULT_CARD_LIMIT`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_limit: Option<u32>,
}

/// Règles sur la carte : le défaut et les bornes (§2.1).
pub const DEFAULT_CARD_LIMIT: usize = 4;
pub const MIN_CARD_LIMIT: u32 = 2;
pub const MAX_CARD_LIMIT: u32 = 6;

impl RulesBundle {
    pub fn lang_code(locale: &str) -> String {
        let trimmed = locale.trim();
        if trimmed.is_empty() {
            return "fr".to_string();
        }
        let lower = trimmed.to_ascii_lowercase();
        let base = lower.split(['-', '_']).next().unwrap_or("fr").trim();
        if base.is_empty() {
            "fr".to_string()
        } else {
            base.to_string()
        }
    }

    pub fn from_row(content_fr: &str, content_en: &str) -> Self {
        let mut bundle = Self::parse_row(content_fr, content_en);
        for payload in bundle.by_lang.values_mut() {
            payload.backfill_ids();
        }
        bundle
    }

    fn parse_row(content_fr: &str, content_en: &str) -> Self {
        if let Ok(value) = serde_json::from_str::<Value>(content_fr.trim()) {
            if value.get("by_lang").is_some() {
                if let Ok(bundle) = serde_json::from_value::<RulesBundle>(value.clone()) {
                    return bundle;
                }
            }
            // Legacy single-locale payload in content_fr.
            if value.get("items").is_some() {
                let mut bundle = RulesBundle::default();
                let fr = RulesPayload::parse(content_fr);
                if !fr.is_empty() {
                    bundle.by_lang.insert("fr".into(), fr);
                }
                let en = RulesPayload::parse(content_en);
                if !en.is_empty() {
                    bundle.by_lang.insert("en".into(), en);
                }
                return bundle;
            }
        }
        let mut bundle = RulesBundle::default();
        let fr = RulesPayload::parse(content_fr);
        if !fr.is_empty() {
            bundle.by_lang.insert("fr".into(), fr);
        }
        let en = RulesPayload::parse(content_en);
        if !en.is_empty() {
            bundle.by_lang.insert("en".into(), en);
        }
        bundle
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Les règles sur la carte, bornées ; le défaut sans choix.
    pub fn card_limit(&self) -> usize {
        self.card_limit.map_or(DEFAULT_CARD_LIMIT, |n| {
            n.clamp(MIN_CARD_LIMIT, MAX_CARD_LIMIT) as usize
        })
    }

    pub fn get(&self, lang: &str) -> RulesPayload {
        self.by_lang
            .get(&Self::lang_code(lang))
            .cloned()
            .unwrap_or_default()
    }

    /// Une langue sans aucune ligne disparaît ; une ligne sans titre reste, pour que le
    /// formulaire la montre avec son erreur.
    pub fn set(&mut self, lang: &str, payload: RulesPayload) {
        let code = Self::lang_code(lang);
        if payload.items.is_empty() {
            self.by_lang.remove(&code);
        } else {
            self.by_lang.insert(code, payload);
        }
    }

    /// Recopie dans chaque langue ce qui n'appartient pas à une langue : l'icône, le statut, le
    /// thème (une clé de la liste) et les heures — d'une règle à celle de même identifiant.
    ///
    /// L'icône ne s'écrase que si la source en porte une — une langue éditée sans icône ne doit pas
    /// effacer celle qui existe. Le statut, lui, se recopie toujours : « neutre » est une valeur que
    /// l'hôte a choisie, et la sauter laisserait une règle importante en français et neutre en
    /// anglais. Un ancien thème libre reste dans sa langue.
    pub fn sync_shared_from(&mut self, source: &RulesPayload) {
        for payload in self.by_lang.values_mut() {
            for item in payload.items.iter_mut() {
                if let Some(src) = source.find(&item.id) {
                    if !src.icon.trim().is_empty() {
                        item.icon = src.icon.clone();
                    }
                    item.status = src.status;
                    if src.theme_key().is_some() {
                        item.theme = src.theme.clone();
                    }
                    item.hours = src.hours.clone();
                }
            }
        }
    }

    /// Après un enregistrement dans une langue : l'ordre reçu vaut pour toutes, et une règle que
    /// l'hôte voyait (`seen`) et n'a pas renvoyée est supprimée partout. Une règle qui n'existe que
    /// dans une autre langue, l'hôte ne la voyait pas : elle reste, après les autres.
    pub fn follow_order(&mut self, source: &RulesPayload, seen: &[String]) {
        let rank = |id: &str| {
            source
                .items
                .iter()
                .position(|item| item.id == id)
                .unwrap_or(usize::MAX)
        };
        for payload in self.by_lang.values_mut() {
            payload
                .items
                .retain(|item| source.find(&item.id).is_some() || !seen.contains(&item.id));
            payload.items.sort_by_key(|item| rank(&item.id));
        }
        self.by_lang.retain(|_, payload| !payload.items.is_empty());
    }

    pub fn pick(&self, guest_locale: &str, property_locale: &str) -> RulesPayload {
        let candidates = [
            Self::lang_code(guest_locale),
            Self::lang_code(property_locale),
            "fr".to_string(),
        ];
        let mut tried = std::collections::BTreeSet::new();
        for lang in &candidates {
            if !tried.insert(lang.clone()) {
                continue;
            }
            let payload = self.get(lang);
            if !payload.is_empty() {
                return payload;
            }
        }
        for payload in self.by_lang.values() {
            if !payload.is_empty() {
                return payload.clone();
            }
        }
        RulesPayload::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le thème (une clé) et les heures se recopient entre les langues ; un ancien thème libre
    /// reste dans la sienne.
    #[test]
    fn shared_fields_follow_the_edited_language() {
        let item = |theme: &str, hours: &str| RuleItem {
            title: "x".into(),
            theme: theme.into(),
            hours: hours.into(),
            ..RuleItem::default()
        };
        let mut bundle = RulesBundle::default();
        bundle.set(
            "en",
            RulesPayload {
                items: vec![item("Neighbours", "")],
                ..RulesPayload::default()
            },
        );
        bundle.sync_shared_from(&RulesPayload {
            items: vec![item("noise", "22:00 – 08:00")],
            ..RulesPayload::default()
        });
        let en = bundle.get("en");
        assert_eq!(en.items[0].theme, "noise");
        assert_eq!(en.items[0].hours, "22:00 – 08:00");
        assert_eq!(en.items[0].theme_title(), "i18n:rule.theme.noise");

        bundle.sync_shared_from(&RulesPayload {
            items: vec![item("Voisinage", "")],
            ..RulesPayload::default()
        });
        assert_eq!(bundle.get("en").items[0].theme, "noise");
        assert_eq!(item("Voisinage", "").theme_title(), "Voisinage");
    }

    /// Les règles sur la carte : le défaut sans choix, bornées sinon ; les longueurs de la spec.
    #[test]
    fn card_limit_and_lengths() {
        let mut bundle = RulesBundle::default();
        assert_eq!(bundle.card_limit(), DEFAULT_CARD_LIMIT);
        bundle.card_limit = Some(9);
        assert_eq!(bundle.card_limit(), MAX_CARD_LIMIT as usize);
        let long = RuleItem {
            title: "x".repeat(61),
            subtitle: "y".repeat(280),
            ..RuleItem::default()
        };
        let fields: Vec<&str> = long.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(fields, ["title"]);
        assert_eq!(RuleStatus::from_wire("forbidden"), RuleStatus::Forbidden);
    }
}
