//! Ce que les checklists savent d'elles-mêmes : les accesseurs de la liste déclarée, et les
//! listes que le livret montre.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::resource::store;
use portaki_sdk::Result;
use uuid::Uuid;

use super::lists;
use crate::{Audience, List, Lists, Step};

/// « Étapes visibles » tant que l'hôte n'a rien choisi.
pub const DEFAULT_VISIBLE_LIMIT: u32 = 5;

impl List {
    pub fn uuid(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap_or_default()
    }

    pub fn is_guest(&self) -> bool {
        self.audience == Audience::Guest
    }

    /// Le moment de la liste, selon qui la coche.
    pub fn trigger(&self) -> &'static str {
        use portaki_sdk::resource::Choice;
        if self.is_guest() {
            self.guest_trigger.wire()
        } else {
            self.host_trigger.wire()
        }
    }

    /// L'échéance d'une tâche d'équipe.
    pub fn deadline(&self) -> Option<&'static str> {
        use portaki_sdk::resource::Choice;
        (!self.is_guest()).then(|| self.deadline.wire())
    }

    /// « Julie Martin · Ménage » : le nom, puis le rôle après le point médian.
    pub fn assignee(&self) -> (Option<String>, Option<String>) {
        let clean = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
        match self.assignee.split_once('·') {
            Some((name, role)) => (clean(name), clean(role)),
            None => (clean(&self.assignee), None),
        }
    }

    pub fn visible_limit(&self) -> u32 {
        self.visible_limit
            .map_or(DEFAULT_VISIBLE_LIMIT, |n| n.clamp(3, 10))
    }

    /// « Rappel la veille du départ » ; absent = oui (spec).
    pub fn remind(&self) -> bool {
        self.remind.unwrap_or(true)
    }

    /// Les étapes écrites : une étape sans texte n'est l'étape de personne.
    pub fn written_steps(&self) -> Vec<Step> {
        self.steps
            .iter()
            .filter(|step| !step.label.is_blank())
            .cloned()
            .collect()
    }
}

impl Step {
    pub fn uuid(&self) -> Uuid {
        Uuid::parse_str(&self.id).unwrap_or_default()
    }

    /// La photo n'est demandée qu'à l'équipe.
    pub fn needs_photo(&self, list: &List) -> bool {
        self.photo && !list.is_guest()
    }
}

/// Le texte dans `locale`, sinon dans la langue du logement, sinon une autre.
pub fn pick(text: &I18nText, locale: &str, property_locale: &str) -> String {
    let short = |l: &str| {
        let lang = l.trim().split(['-', '_']).next().unwrap_or_default();
        lang.to_ascii_lowercase()
    };
    let wanted = [short(locale), short(property_locale)];
    wanted
        .iter()
        .find_map(|lang| {
            text.by_language()
                .find(|(l, t)| l.eq_ignore_ascii_case(lang) && !t.trim().is_empty())
                .map(|(_, t)| t.trim().to_string())
        })
        .unwrap_or_else(|| text.get(locale).trim().to_string())
}

/// Les listes du logement, dans l'ordre de l'hôte.
pub fn all_lists() -> Result<Vec<List>> {
    Ok(
        store::load::<List, portaki_sdk::resource::NoMeta>(Lists::DECL)?
            .rows()
            .to_vec(),
    )
}

/// Les listes du livret : toutes celles du voyageur, mais une seule liste de départ — la première
/// (« Une seule checklist « Voyageur · Départ » à la fois »).
pub fn booklet_lists() -> Result<Vec<List>> {
    let mut departure_seen = false;
    Ok(all_lists()?
        .into_iter()
        .filter(|list| {
            list.is_guest()
                && (!lists::is_departure_trigger(list.trigger())
                    || !std::mem::replace(&mut departure_seen, true))
        })
        .collect())
}

/// Les étapes des listes que le livret montre ([`crate::domain::booklet_lists`]), liste par liste.
pub fn guest_items() -> Result<Vec<Step>> {
    Ok(booklet_lists()?
        .iter()
        .flat_map(List::written_steps)
        .collect())
}

/// L'étape `id`, et sa liste.
pub fn find_step(id: Uuid) -> Result<Option<(List, Step)>> {
    Ok(all_lists()?.into_iter().find_map(|list| {
        let step = list.written_steps().into_iter().find(|s| s.uuid() == id)?;
        Some((list, step))
    }))
}
