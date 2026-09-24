//! Checklist wire values and the four templates a new list starts from.

use crate::i18n;

pub const GUEST: &str = "guest";
pub const HOST: &str = "host";

/// Guest triggers: when the list shows in the booklet.
pub const BEFORE_ARRIVAL: &str = "beforeArrival";
pub const DURING_STAY: &str = "duringStay";
pub const AT_DEPARTURE: &str = "atDeparture";
pub const GUEST_TRIGGERS: &[&str] = &[BEFORE_ARRIVAL, DURING_STAY, AT_DEPARTURE];

/// Guest placement: the booklet, or the booklet plus an e-mail the day before departure.
pub const BOOKLET: &str = "booklet";
pub const BOOKLET_EMAIL: &str = "booklet+email";
pub const PLACEMENTS: &[&str] = &[BOOKLET, BOOKLET_EMAIL];

/// Host triggers: which stays get a task.
pub const BEFORE_EACH_ARRIVAL: &str = "beforeEachArrival";
pub const AT_DEPARTURE_BEFORE_CLEANING: &str = "atDepartureBeforeCleaning";
pub const AFTER_EACH_DEPARTURE: &str = "afterEachDeparture";
pub const ONLY_IF_NEXT_ARRIVAL: &str = "onlyIfNextArrival";
pub const HOST_TRIGGERS: &[&str] = &[
    BEFORE_EACH_ARRIVAL,
    AT_DEPARTURE_BEFORE_CLEANING,
    AFTER_EACH_DEPARTURE,
    ONLY_IF_NEXT_ARRIVAL,
];

/// Host deadlines.
pub const ARRIVAL: &str = "arrival";
pub const NEXT_ARRIVAL: &str = "nextArrival";
pub const NEXT_ARRIVAL_MINUS_2H: &str = "nextArrivalMinus2h";
pub const DEPARTURE_EVENING: &str = "departureEvening";
pub const DEADLINES: &[&str] = &[
    ARRIVAL,
    NEXT_ARRIVAL,
    NEXT_ARRIVAL_MINUS_2H,
    DEPARTURE_EVENING,
];

/// Returns `raw` when it is one of `allowed`, else the first allowed value.
pub fn pick(raw: &str, allowed: &[&'static str]) -> &'static str {
    allowed
        .iter()
        .copied()
        .find(|value| *value == raw.trim())
        .unwrap_or(allowed[0])
}

/// A list to start from. Labels come from the module bundles (`template.<id>.*`).
pub struct Template {
    pub id: &'static str,
    pub audience: &'static str,
    pub icon: &'static str,
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
        icon: "logout",
        trigger: AT_DEPARTURE,
        deadline: None,
        items: 5,
        photo: None,
    },
    Template {
        id: "cleaning",
        audience: HOST,
        icon: "sparkles",
        trigger: AFTER_EACH_DEPARTURE,
        deadline: Some(NEXT_ARRIVAL_MINUS_2H),
        items: 7,
        photo: Some(6),
    },
    Template {
        id: "checkIn",
        audience: HOST,
        icon: "home",
        trigger: BEFORE_EACH_ARRIVAL,
        deadline: Some(ARRIVAL),
        items: 5,
        photo: Some(4),
    },
    Template {
        id: "checkOut",
        audience: HOST,
        icon: "search",
        trigger: AT_DEPARTURE_BEFORE_CLEANING,
        deadline: Some(DEPARTURE_EVENING),
        items: 4,
        photo: Some(2),
    },
    Template {
        id: "emptyGuest",
        audience: GUEST,
        icon: "check-circle",
        trigger: AT_DEPARTURE,
        deadline: None,
        items: 0,
        photo: None,
    },
    Template {
        id: "emptyHost",
        audience: HOST,
        icon: "clipboard",
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
        let text = i18n::text(&format!("template.{}.name", self.id), &[]);
        (text.fr, text.en)
    }

    /// `(fr, en, photo required)` of each item.
    pub fn item_labels(&self) -> Vec<(String, String, bool)> {
        (0..self.items)
            .map(|index| {
                let text = i18n::text(&format!("template.{}.item.{index}", self.id), &[]);
                (text.fr, text.en, self.photo == Some(index))
            })
            .collect()
    }
}

/// Ticking this item of a host list means the consumables were restocked.
pub fn is_restock_label(fr: &str, en: &str) -> bool {
    let restock = i18n::text("template.cleaning.item.5", &[]);
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
        let cleaning = template("cleaning").unwrap().item_labels();
        assert_eq!(cleaning.len(), 7);
        assert!(cleaning[6].2);
        assert!(is_restock_label(&cleaning[5].0, ""));
    }

    #[test]
    fn pick_falls_back_to_first_value() {
        assert_eq!(pick("atDeparture", GUEST_TRIGGERS), AT_DEPARTURE);
        assert_eq!(pick("bogus", GUEST_TRIGGERS), BEFORE_ARRIVAL);
    }
}
