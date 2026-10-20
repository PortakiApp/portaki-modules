//! Les tâches de ménage telles que les statistiques les lisent : coches, échéance, retard.

#[allow(unused_imports, reason = "the parent's imports, shared by its parts")]
use super::*;

/// Ticks of one task of a host cleaning list.
pub(super) struct CleaningTask<'a> {
    pub(super) list: &'a List,
    pub(super) items: Vec<&'a Step>,
    pub(super) done: Vec<&'a TaskItemState>,
    /// `(departure, deadline)` when the platform passed the stays.
    pub(super) plan: Option<(DateTime<Utc>, Option<DateTime<Utc>>)>,
}

/// Where a cleaning stands against its deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Timing {
    Open,
    OnTime,
    /// Finished after its deadline, or not finished once it has passed.
    Late,
}

impl CleaningTask<'_> {
    pub(super) fn finished(&self) -> bool {
        self.done.len() >= self.items.len()
    }

    pub(super) fn deadline(&self) -> Option<DateTime<Utc>> {
        self.plan.and_then(|(_, due)| due)
    }

    pub(super) fn timing(&self, now: DateTime<Utc>) -> Timing {
        match (self.finished(), self.deadline(), self.last_at()) {
            (true, Some(due), Some(at)) if at > due => Timing::Late,
            (true, _, _) => Timing::OnTime,
            (false, Some(due), _) if now > due => Timing::Late,
            (false, _, _) => Timing::Open,
        }
    }

    pub(super) fn first_at(&self) -> Option<DateTime<Utc>> {
        self.done.iter().filter_map(|state| state.done_at).min()
    }

    pub(super) fn last_at(&self) -> Option<DateTime<Utc>> {
        self.done.iter().filter_map(|state| state.done_at).max()
    }

    pub(super) fn photos(&self) -> usize {
        self.done
            .iter()
            .filter(|state| state.photo.is_some())
            .count()
    }
}

/// Everything `CleaningTask` borrows from, loaded once.
pub(super) struct CleaningData {
    pub(super) lists: Vec<List>,
    /// Every step, with the id of its list.
    pub(super) items: Vec<(Uuid, Step)>,
    pub(super) states: Vec<TaskItemState>,
}

impl CleaningData {
    pub(super) fn steps_of(&self, list: &List) -> Vec<&Step> {
        let id = list.uuid();
        self.items
            .iter()
            .filter(|(owner, _)| *owner == id)
            .map(|(_, step)| step)
            .collect()
    }
}

pub(super) fn cleaning_data() -> Result<CleaningData> {
    let lists: Vec<List> = crate::domain::all_lists()?
        .into_iter()
        .filter(|list| {
            !list.is_guest()
                && [lists::AFTER_EACH_DEPARTURE, lists::ONLY_IF_NEXT_ARRIVAL]
                    .contains(&list.trigger())
        })
        .collect();
    let items = lists
        .iter()
        .flat_map(|list| {
            let id = list.uuid();
            list.written_steps().into_iter().map(move |step| (id, step))
        })
        .collect();
    Ok(CleaningData {
        lists,
        items,
        states: storage::task_states(None)?,
    })
}

/// Tasks ticked since `since`, most recent activity first.
pub(super) fn cleaning_tasks_of(
    data: &CleaningData,
    since: DateTime<Utc>,
) -> Vec<CleaningTask<'_>> {
    let mut by_task: BTreeMap<&str, CleaningTask<'_>> = BTreeMap::new();
    for state in data.states.iter().filter(|state| state.done) {
        let Some(list) = parse_task_id(&state.task_id)
            .and_then(|(list_id, _)| data.lists.iter().find(|list| list.uuid() == list_id))
        else {
            continue;
        };
        let task = by_task
            .entry(&state.task_id)
            .or_insert_with(|| CleaningTask {
                list,
                items: data.steps_of(list),
                done: Vec::new(),
                plan: None,
            });
        if task.items.iter().any(|item| item.uuid() == state.item_id) {
            task.done.push(state);
        }
    }
    let mut tasks: Vec<CleaningTask<'_>> = by_task
        .into_values()
        .filter(|task| task.last_at().is_some_and(|at| at >= since))
        .collect();
    tasks.sort_by_key(|task| std::cmp::Reverse(task.last_at()));
    tasks
}

/// One task per departure of `(since, now]` the list's trigger applies to — ticked or not — with
/// its deadline, most recent departure first.
pub(super) fn planned_cleanings<'a>(
    data: &'a CleaningData,
    stays: &[TimelineStay],
    timezone: &str,
    since: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Vec<CleaningTask<'a>> {
    let mut tasks = Vec::new();
    for list in &data.lists {
        let items = data.steps_of(list);
        if items.is_empty() {
            continue;
        }
        for plan in plan_tasks(list, stays, timezone) {
            if plan.at <= since || plan.at > now {
                continue;
            }
            let id = task_id(list.uuid(), plan.stay_id);
            let done = data
                .states
                .iter()
                .filter(|state| {
                    state.done
                        && state.task_id == id
                        && items.iter().any(|item| item.uuid() == state.item_id)
                })
                .collect();
            tasks.push(CleaningTask {
                list,
                items: items.clone(),
                done,
                plan: Some((plan.at, plan.due_at)),
            });
        }
    }
    tasks.sort_by_key(|task| std::cmp::Reverse(task.plan.map(|(at, _)| at)));
    tasks
}
