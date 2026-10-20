//! Ce que les checklists savent d'elles-mêmes : les accesseurs de la liste déclarée, les listes
//! que le livret montre, les modèles, et la règle de publication qu'aucun champ ne dit.

pub mod entities;
pub mod ids;
pub mod lists;
pub mod show_when;
pub mod storage;

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::resource::{store, Collection, Problem};
use uuid::Uuid;

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

/// L'étape `id`, et sa liste.
pub fn find_step(id: Uuid) -> Result<Option<(List, Step)>> {
    Ok(all_lists()?.into_iter().find_map(|list| {
        let step = list.written_steps().into_iter().find(|s| s.uuid() == id)?;
        Some((list, step))
    }))
}

/// Une liste tirée d'un modèle, ajoutée après les autres.
pub fn add_from_template(template: &lists::Template, id: Option<Uuid>) -> Result<List> {
    let mut lists = store::load::<List, portaki_sdk::resource::NoMeta>(Lists::DECL)?;
    let (fr, en) = template.name();
    let guest = template.audience == lists::GUEST;
    let groups = template.item_groups();
    let steps = template
        .item_labels()
        .into_iter()
        .enumerate()
        .map(|(index, (fr, en, photo))| Step {
            id: Uuid::new_v4().to_string(),
            label: I18nText::new(fr, en),
            zone: groups
                .get(index)
                .map(|(fr, en)| I18nText::new(fr.clone(), en.clone()))
                .unwrap_or_default(),
            photo,
            ..Step::default()
        })
        .collect();
    let mut list = List {
        id: id.unwrap_or_else(Uuid::new_v4).to_string(),
        name: I18nText::new(fr, en),
        audience: if guest {
            Audience::Guest
        } else {
            Audience::Host
        },
        icon: template.icon.to_string(),
        notify_assignee: !guest,
        alert_host: !guest,
        steps,
        ..List::default()
    };
    set_moments(&mut list, template);
    lists.rows_mut().push(list.clone());
    Lists::save(&lists)?;
    Ok(list)
}

fn set_moments(list: &mut List, template: &lists::Template) {
    use portaki_sdk::resource::Choice;
    if let Some(trigger) = crate::GuestTrigger::from_wire(template.trigger) {
        list.guest_trigger = trigger;
    }
    if let Some(trigger) = crate::HostTrigger::from_wire(template.trigger) {
        list.host_trigger = trigger;
    }
    if let Some(deadline) = template.deadline.and_then(crate::Deadline::from_wire) {
        list.deadline = deadline;
    }
}

/// Une étape écrite au moins, sur l'ensemble des listes.
pub fn problems(lists: &Collection<List, portaki_sdk::resource::NoMeta>) -> Vec<Problem> {
    let written = lists
        .rows()
        .iter()
        .any(|list| !list.written_steps().is_empty());
    (!written)
        .then(|| Problem {
            row: None,
            field: "steps",
            message: portaki_sdk::bundle_text!("publish.checklist.hint"),
        })
        .into_iter()
        .collect()
}
