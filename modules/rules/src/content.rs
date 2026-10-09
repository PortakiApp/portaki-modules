//! Structured house-rules payload (not TipTap).

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

    /// Ordre du tri de la carte : interdit et important, puis autorisé, puis le reste (§2.8).
    pub fn rank(self) -> u8 {
        match self {
            Self::Forbidden | Self::Important => 0,
            Self::Allowed => 1,
            Self::Neutral => 2,
        }
    }
}

/// One rule row shown in the guest booklet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuleItem {
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

impl RuleItem {
    /// Le thème dans la liste, ou `None` pour un ancien texte libre (ou rien).
    pub fn theme_key(&self) -> Option<&'static str> {
        THEMES
            .iter()
            .find(|key| **key == self.theme.trim())
            .copied()
    }

    /// Les longueurs de la spec (§2.2), par champ du formulaire : titre ≤ 60, précision ≤ 280.
    pub fn problems(&self) -> Vec<(&'static str, portaki_sdk::contracts::i18n::I18nText)> {
        use portaki_sdk::config::check::max_chars;
        [
            ("title", max_chars(&self.title, TITLE_MAX)),
            ("subtitle", max_chars(&self.subtitle, DETAIL_MAX)),
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

    /// Triées important → autorisé → neutres (§2.8). Tri **stable** : à statut égal, l'ordre que
    /// l'hôte a choisi tient. Ne sert qu'au coup d'œil de la carte ; le détail garde l'ordre brut.
    pub fn by_weight(&self) -> Vec<&RuleItem> {
        let mut out: Vec<&RuleItem> = self.named().collect();
        out.sort_by_key(|item| item.status.rank());
        out
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

    pub fn set(&mut self, lang: &str, payload: RulesPayload) {
        let code = Self::lang_code(lang);
        if payload.is_empty() {
            self.by_lang.remove(&code);
        } else {
            self.by_lang.insert(code, payload);
        }
    }

    /// Recopie dans chaque langue ce qui n'appartient pas à une langue : l'icône, le statut, le
    /// thème (une clé de la liste) et les heures.
    ///
    /// L'icône ne s'écrase que si la source en porte une — une langue éditée sans icône ne doit pas
    /// effacer celle qui existe. Le statut, lui, se recopie toujours : « neutre » est une valeur que
    /// l'hôte a choisie, et la sauter laisserait une règle importante en français et neutre en
    /// anglais. Le thème n'est pas ici : c'est un texte, il reste dans sa langue.
    ///
    /// ponytail: l'alignement se fait par index, après que `build_payload_for_lang` ait écarté les
    /// lignes sans titre — vider la règle 2 en français décale donc l'anglais. Défaut antérieur, à
    /// corriger en donnant un identifiant stable à chaque règle.
    pub fn sync_shared_from(&mut self, source: &RulesPayload) {
        for payload in self.by_lang.values_mut() {
            for (index, item) in payload.items.iter_mut().enumerate() {
                if let Some(src) = source.items.get(index) {
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
