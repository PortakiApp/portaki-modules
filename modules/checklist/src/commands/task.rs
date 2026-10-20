//! What the three task operations and the host stats share: where a task sits, its id, its items,
//! and what a change emits.
//!
//! A task is computed from a host list and a stay the platform passes; only the ticked state is
//! stored (`task_item_state`). Its id is `<checklistId>:<stayId>`.

use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{
    self, TaskUpdated, TimelineStay, TimelineTask, TimelineTaskItem, TASK_UPDATED,
};
use portaki_sdk::host::events;
use portaki_sdk::prelude::*;
use serde_json::json;
use uuid::Uuid;

use super::templates::is_restock_label;
use crate::domain::timezone::offset_for_iana;
use crate::TaskItemState;
use crate::{lists, storage, List, Step};

/// Where a task of `list` sits for one stay, before its items are attached.
#[derive(Debug, PartialEq)]
pub struct TaskPlan {
    pub stay_id: Uuid,
    pub at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub context: I18nText,
}

/// One plan per stay the list's trigger applies to.
pub fn plan_tasks(list: &List, stays: &[TimelineStay], timezone: &str) -> Vec<TaskPlan> {
    let live: Vec<&TimelineStay> = stays.iter().filter(|stay| !is_cancelled(stay)).collect();
    let mut plans = Vec::new();
    for (index, stay) in live.iter().enumerate() {
        let next = live[index + 1..]
            .iter()
            .find(|next| next.check_in >= stay.check_out);
        let arrival_trigger = list.trigger() == lists::BEFORE_EACH_ARRIVAL;
        if list.trigger() == lists::ONLY_IF_NEXT_ARRIVAL && next.is_none() {
            continue;
        }
        // The arrival a deadline refers to: this stay's for an arrival task, else the next one.
        let arrival = if arrival_trigger {
            Some(stay.check_in)
        } else {
            next.map(|next| next.check_in)
        };
        let due_at = match list.deadline() {
            Some(lists::ARRIVAL | lists::NEXT_ARRIVAL) => arrival,
            Some(lists::NEXT_ARRIVAL_MINUS_2H) => arrival.map(|at| at - Duration::hours(2)),
            Some(lists::DEPARTURE_EVENING) => Some(evening_of(stay.check_out, timezone)),
            _ => None,
        };
        let context = match (arrival_trigger, next) {
            (true, _) => portaki_sdk::bundle_text!(
                "task.context.beforeArrival",
                &[("guest", &stay.guest_name)]
            ),
            (false, Some(next)) if list.trigger() != lists::AT_DEPARTURE_BEFORE_CLEANING => {
                portaki_sdk::bundle_text!(
                    "task.context.beforeArrival",
                    &[("guest", &next.guest_name)]
                )
            }
            _ => portaki_sdk::bundle_text!(
                "task.context.afterDeparture",
                &[("guest", &stay.guest_name)],
            ),
        };
        plans.push(TaskPlan {
            stay_id: stay.id,
            at: if arrival_trigger {
                stay.check_in
            } else {
                stay.check_out
            },
            due_at,
            context,
        });
    }
    plans
}

fn is_cancelled(stay: &TimelineStay) -> bool {
    stay.status.to_ascii_uppercase().starts_with("CANCEL")
}

/// 20:00 on the day of `at`, property time.
fn evening_of(at: DateTime<Utc>, timezone: &str) -> DateTime<Utc> {
    let offset = offset_for_iana(timezone, at);
    let evening = at
        .with_timezone(&offset)
        .date_naive()
        .and_time(NaiveTime::from_hms_opt(20, 0, 0).unwrap_or_default());
    offset
        .from_local_datetime(&evening)
        .single()
        .map_or(at, |local| local.with_timezone(&Utc))
}

pub(super) fn build_task(
    property_id: Uuid,
    list: &List,
    items: &[Step],
    states: &[TaskItemState],
    plan: TaskPlan,
) -> TimelineTask {
    let task_id = task_id(list.uuid(), plan.stay_id);
    let mut task = timeline::task(
        task_id.clone(),
        plan.at,
        property_id,
        list.name.clone(),
        plan.context,
    )
    .stay(plan.stay_id)
    .items(
        items
            .iter()
            .map(|item| task_item(list, item, state_of(states, &task_id, item.uuid())))
            .collect(),
    );
    if let Some(due_at) = plan.due_at {
        task = task.due_at(due_at);
    }
    if let (Some(name), role) = list.assignee() {
        let role = role.unwrap_or_default();
        task = task.assignee(&name, I18nText::new(role.clone(), role));
    }
    task
}

pub(super) fn state_of<'a>(
    states: &'a [TaskItemState],
    task_id: &str,
    item_id: Uuid,
) -> Option<&'a TaskItemState> {
    states
        .iter()
        .find(|state| state.task_id == task_id && state.item_id == item_id)
}

pub(super) fn task_item(
    list: &List,
    item: &Step,
    state: Option<&TaskItemState>,
) -> TimelineTaskItem {
    let mut out = TimelineTaskItem::new(item.id.clone(), item.label.clone());
    out.photo_required = item.needs_photo(list);
    if let Some(state) = state.filter(|state| state.done) {
        out.done = true;
        out.photo = state.photo.clone();
    }
    out
}

pub fn task_id(checklist_id: Uuid, stay_id: Uuid) -> String {
    format!("{checklist_id}:{stay_id}")
}

/// `(checklist, stay)` of a task id.
pub fn parse_task_id(task_id: &str) -> Option<(Uuid, Uuid)> {
    let (list, stay) = task_id.split_once(':')?;
    Some((Uuid::parse_str(list).ok()?, Uuid::parse_str(stay).ok()?))
}

pub(super) fn load_task(ctx: &Context, task_id: &str) -> Result<(List, Vec<Step>, Uuid)> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }
    let not_found = || PortakiError::Host("task_not_found".to_string());
    let (list_id, stay_id) = parse_task_id(task_id).ok_or_else(not_found)?;
    let list = crate::domain::all_lists()?
        .into_iter()
        .find(|list| list.uuid() == list_id && !list.is_guest())
        .ok_or_else(not_found)?;
    let items = list.written_steps();
    Ok((list, items, stay_id))
}

pub(super) fn is_restock(item: &Step) -> bool {
    is_restock_label(&item.label.fr, &item.label.en)
}

pub(super) fn after_change(
    ctx: &Context,
    list: &List,
    task_id: &str,
    stay_id: Uuid,
    restocked: bool,
) -> Result<()> {
    let items = list.written_steps();
    let states = storage::task_states(Some(task_id))?;
    let done = items
        .iter()
        .filter(|item| state_of(&states, task_id, item.uuid()).is_some_and(|s| s.done))
        .count() as u32;
    let total = items.len() as u32;
    events::emit(
        TASK_UPDATED,
        &TaskUpdated {
            property_id: ctx.property_id,
            stay_id: Some(stay_id),
            task_id: task_id.to_string(),
            done,
            total,
            assignee_name: list.assignee().0,
        },
    )?;
    let progress = format!("{done}/{total}");
    events::emit(
        crate::ids::WORKSPACE_ACTIVITY_RECORD,
        &json!({
            "eventId": TASK_UPDATED.as_str(),
            "propertyId": ctx.property_id,
            "payload": { "taskId": task_id, "stayId": stay_id, "done": done, "total": total },
            "display": { "chips": [
                { "label": list.name.get("fr"), "value": progress },
            ] },
        }),
    )?;
    if restocked {
        events::emit(
            crate::ids::CONSUMABLES_RESTOCKED,
            &json!({ "propertyId": ctx.property_id, "stayId": stay_id }),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn stay(guest: &str, check_in: &str, check_out: &str) -> TimelineStay {
        TimelineStay {
            id: Uuid::new_v4(),
            check_in: at(check_in),
            check_out: at(check_out),
            guest_name: guest.into(),
            status: "UPCOMING".into(),
        }
    }

    fn list(trigger: &str, deadline: Option<&str>) -> List {
        use portaki_sdk::resource::Choice;
        List {
            id: Uuid::new_v4().to_string(),
            name: I18nText::new("Ménage", "Cleaning"),
            audience: crate::Audience::Host,
            host_trigger: crate::HostTrigger::from_wire(trigger).unwrap(),
            deadline: deadline
                .and_then(crate::Deadline::from_wire)
                .unwrap_or_default(),
            ..List::default()
        }
    }

    #[test]
    fn cleaning_sits_at_departure_and_ends_two_hours_before_next_arrival() {
        let stays = [
            stay("Marie", "2026-08-12T15:00:00Z", "2026-08-19T09:00:00Z"),
            stay("Liam", "2026-08-21T14:00:00Z", "2026-08-25T09:00:00Z"),
        ];
        let plans = plan_tasks(
            &list(
                lists::AFTER_EACH_DEPARTURE,
                Some(lists::NEXT_ARRIVAL_MINUS_2H),
            ),
            &stays,
            "UTC",
        );
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].at, at("2026-08-19T09:00:00Z"));
        assert_eq!(plans[0].due_at, Some(at("2026-08-21T12:00:00Z")));
        assert!(plans[0].context.fr.contains("Liam"));
        assert_eq!(plans[1].due_at, None);
        assert!(plans[1].context.fr.contains("Liam"));
    }

    #[test]
    fn only_if_next_arrival_skips_the_last_stay_and_cancelled_ones() {
        let mut cancelled = stay("Tom", "2026-08-20T14:00:00Z", "2026-08-22T09:00:00Z");
        cancelled.status = "CANCELLED".into();
        let stays = [
            stay("Marie", "2026-08-12T15:00:00Z", "2026-08-19T09:00:00Z"),
            cancelled,
        ];
        let plans = plan_tasks(&list(lists::ONLY_IF_NEXT_ARRIVAL, None), &stays, "UTC");
        assert!(plans.is_empty());
    }

    #[test]
    fn arrival_task_is_due_at_check_in_and_departure_check_in_the_evening() {
        let stays = [stay(
            "Sofia",
            "2026-08-02T15:00:00Z",
            "2026-08-09T08:00:00Z",
        )];
        let arrival = plan_tasks(
            &list(lists::BEFORE_EACH_ARRIVAL, Some(lists::ARRIVAL)),
            &stays,
            "UTC",
        );
        assert_eq!(arrival[0].at, at("2026-08-02T15:00:00Z"));
        assert_eq!(arrival[0].due_at, Some(at("2026-08-02T15:00:00Z")));
        let departure = plan_tasks(
            &list(
                lists::AT_DEPARTURE_BEFORE_CLEANING,
                Some(lists::DEPARTURE_EVENING),
            ),
            &stays,
            "Europe/Paris",
        );
        assert_eq!(departure[0].due_at, Some(at("2026-08-09T18:00:00Z")));
    }

    #[test]
    fn task_ids_round_trip() {
        let (list, stay) = (Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(parse_task_id(&task_id(list, stay)), Some((list, stay)));
        assert_eq!(parse_task_id("cleaning"), None);
    }
}
