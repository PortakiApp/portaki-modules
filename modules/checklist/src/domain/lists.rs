//! Checklist wire values and the templates a new list starts from.

pub const GUEST: &str = "guest";
pub const HOST: &str = "host";

/// Guest triggers (« Afficher », spec Checklist §2.1): when the list shows in the booklet —
/// « Pendant tout le séjour », « La veille du départ », « Le jour du départ ».
///
/// `atDeparture` keeps its wire value for « La veille » so stored lists need no rewrite.
pub const DURING_STAY: &str = "duringStay";
pub const AT_DEPARTURE: &str = "atDeparture";
pub const DEPARTURE_DAY: &str = "departureDay";

/// The lists the guest ticks on their way out — also shown on the post-stay card.
pub fn is_departure_trigger(trigger: &str) -> bool {
    trigger == AT_DEPARTURE || trigger == DEPARTURE_DAY
}

/// Host triggers: which stays get a task.
pub const BEFORE_EACH_ARRIVAL: &str = "beforeEachArrival";
pub const AT_DEPARTURE_BEFORE_CLEANING: &str = "atDepartureBeforeCleaning";
pub const AFTER_EACH_DEPARTURE: &str = "afterEachDeparture";
pub const ONLY_IF_NEXT_ARRIVAL: &str = "onlyIfNextArrival";

/// Host deadlines.
pub const ARRIVAL: &str = "arrival";
pub const NEXT_ARRIVAL: &str = "nextArrival";
pub const NEXT_ARRIVAL_MINUS_2H: &str = "nextArrivalMinus2h";
pub const DEPARTURE_EVENING: &str = "departureEvening";

/// A list to start from. Labels come from the module bundles (`template.<id>.*`).
pub struct Template {
    pub id: &'static str,
    pub audience: &'static str,
    pub icon: portaki_sdk::vocab::IconName,
    pub trigger: &'static str,
    pub deadline: Option<&'static str>,
    pub items: usize,
    /// Index of the item that needs a photo.
    pub photo: Option<usize>,
}

pub const TEMPLATES: &[Template] = &[
    Template {
        id: "departure",
        audience: GUEST,
        icon: portaki_sdk::vocab::IconName::Logout,
        trigger: AT_DEPARTURE,
        deadline: None,
        items: 5,
        photo: None,
    },
    Template {
        id: "arrival",
        audience: GUEST,
        icon: portaki_sdk::vocab::IconName::Key,
        trigger: DURING_STAY,
        deadline: None,
        items: 3,
        photo: None,
    },
    Template {
        id: "cleaning",
        audience: HOST,
        icon: portaki_sdk::vocab::IconName::Sparkles,
        trigger: AFTER_EACH_DEPARTURE,
        deadline: Some(NEXT_ARRIVAL_MINUS_2H),
        items: 7,
        photo: Some(6),
    },
    Template {
        id: "checkIn",
        audience: HOST,
        icon: portaki_sdk::vocab::IconName::Home,
        trigger: BEFORE_EACH_ARRIVAL,
        deadline: Some(ARRIVAL),
        items: 5,
        photo: Some(4),
    },
    Template {
        id: "checkOut",
        audience: HOST,
        icon: portaki_sdk::vocab::IconName::Search,
        trigger: AT_DEPARTURE_BEFORE_CLEANING,
        deadline: Some(DEPARTURE_EVENING),
        items: 4,
        photo: Some(2),
    },
    Template {
        id: "emptyGuest",
        audience: GUEST,
        icon: portaki_sdk::vocab::IconName::CheckCircle,
        trigger: AT_DEPARTURE,
        deadline: None,
        items: 0,
        photo: None,
    },
    Template {
        id: "emptyHost",
        audience: HOST,
        icon: portaki_sdk::vocab::IconName::Clipboard,
        trigger: AFTER_EACH_DEPARTURE,
        deadline: Some(NEXT_ARRIVAL),
        items: 0,
        photo: None,
    },
];

pub fn template(id: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|template| template.id == id)
}

impl Template {
    /// `(fr, en)` of the list name.
    pub fn name(&self) -> (String, String) {
        let text = portaki_sdk::bundle_text!(&format!("template.{}.name", self.id), &[]);
        (text.fr, text.en)
    }

    /// `(fr, en, photo required)` of each item.
    pub fn item_labels(&self) -> Vec<(String, String, bool)> {
        (0..self.items)
            .map(|index| {
                let text =
                    portaki_sdk::bundle_text!(&format!("template.{}.item.{index}", self.id), &[]);
                (text.fr, text.en, self.photo == Some(index))
            })
            .collect()
    }

    /// La rubrique de chaque étape (§2.9), vide quand le modèle n'en range aucune.
    ///
    /// Seule la liste de départ en pose : c'est celle que le voyageur lit, et le groupement n'a de
    /// sens que là. Deux rubriques pour cinq étapes — ce qu'on fait dans le logement, puis ce qu'on
    /// fait en partant — plutôt qu'une par étape, qui n'aurait rien regroupé.
    pub fn item_groups(&self) -> Vec<(String, String)> {
        (0..self.items)
            .map(|index| match self.group_key(index) {
                Some(key) => {
                    let text = portaki_sdk::bundle_text!(key, &[]);
                    (text.fr, text.en)
                }
                None => (String::new(), String::new()),
            })
            .collect()
    }

    fn group_key(&self, index: usize) -> Option<&'static str> {
        if self.id != "departure" {
            return None;
        }
        Some(match index {
            0 | 1 | 4 => "template.departure.group.home",
            _ => "template.departure.group.leaving",
        })
    }
}

/// Ticking this item of a host list means the consumables were restocked.
pub fn is_restock_label(fr: &str, en: &str) -> bool {
    let restock = portaki_sdk::bundle_text!("template.cleaning.item.5", &[]);
    fr.trim().eq_ignore_ascii_case(&restock.fr) || en.trim().eq_ignore_ascii_case(&restock.en)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_have_every_label_in_both_languages() {
        for template in TEMPLATES {
            let (fr, en) = template.name();
            assert!(!fr.is_empty() && !en.is_empty(), "{}", template.id);
            for (fr, en, _) in template.item_labels() {
                assert!(!fr.is_empty() && !en.is_empty(), "{}", template.id);
            }
        }
        assert_eq!(template("arrival").unwrap().item_labels().len(), 3);
        let cleaning = template("cleaning").unwrap().item_labels();
        assert_eq!(cleaning.len(), 7);
        assert!(cleaning[6].2);
        assert!(is_restock_label(&cleaning[5].0, ""));
    }
}
