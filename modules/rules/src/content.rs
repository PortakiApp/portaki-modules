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
    #[default]
    Neutral,
    Important,
    Allowed,
}

impl RuleStatus {
    /// Lit un statut venu du formulaire hôte ou d'un payload stocké. `ok` est l'orthographe du
    /// mockup pour `allowed` ; tout le reste retombe sur neutre plutôt que d'échouer.
    pub fn from_wire(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "important" => Self::Important,
            "allowed" | "ok" => Self::Allowed,
            _ => Self::Neutral,
        }
    }

    /// Le nom qui part dans le formulaire hôte (valeur du `Select`).
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Important => "important",
            Self::Allowed => "allowed",
        }
    }

    /// Ordre du tri de la carte : important, puis autorisé, puis le reste (§2.8).
    pub fn rank(self) -> u8 {
        match self {
            Self::Important => 0,
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
    /// Thème libre, écrit par l'hôte **dans sa langue** : le mockup groupe « Voisinage »,
    /// « Piscine », « Animaux »… C'est un texte, il vit donc dans le payload de sa langue, à côté
    /// du titre, et ne se recopie pas.
    #[serde(default)]
    pub theme: String,
}

/// Locale payload for one language.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RulesPayload {
    #[serde(default)]
    pub items: Vec<RuleItem>,
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
            let label = item.theme.trim();
            let key = label.to_lowercase();
            match groups.iter_mut().find(|(k, _, _)| *k == key) {
                Some((_, _, rules)) => rules.push(item),
                None => groups.push((key, label.to_string(), vec![item])),
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
}

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

    /// Recopie dans chaque langue ce qui n'appartient pas à une langue : l'icône et le statut.
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
