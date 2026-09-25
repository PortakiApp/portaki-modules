//! Host lists as dated tasks: `timelineTasks`, `taskToggle`, `taskComplete`.
//!
//! A task is computed from a host list and a stay the platform passes; only the ticked state is
//! stored (`task_item_state`). Its id is `<checklistId>:<stayId>`.

use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::contracts::timeline::{
    self, TaskCompleteArgs, TaskToggleArgs, TaskUpdated, TimelineStay, TimelineTask,
    TimelineTaskItem, TimelineTasks, TimelineTasksArgs, TASK_UPDATED,
};
use portaki_sdk::files::FileRef;
use portaki_sdk::host::{events, time};
use portaki_sdk::prelude::*;
use serde_json::json;
use uuid::Uuid;

use crate::entities::{Checklist, ChecklistItem, TaskItemState};
use crate::guest::depart::offset_for_iana;
use crate::labels;
use crate::lists;
use crate::{i18n, storage};

#[portaki_sdk::query(
    name = "timelineTasks",
    example(
        label = "Semaine avec un départ",
        input = r#"{"propertyId":"5f0c2b1e-8a4d-4c6f-9e21-3b7d9a6c4e10","from":"2026-06-15T00:00:00Z","to":"2026-06-22T00:00:00Z","stays":[{"id":"8d3e6f2a-1b4c-4d5e-9f60-7a8b9c0d1e2f","checkIn":"2026-06-12T14:00:00Z","checkOut":"2026-06-17T09:00:00Z","guestName":"Camille Durand","status":"ACTIVE"},{"id":"2c4e6a8b-0d1f-4a3c-8e5b-7d9f1a3c5e70","checkIn":"2026-06-19T14:00:00Z","checkOut":"2026-06-26T09:00:00Z","guestName":"Lucas Martin","status":"UPCOMING"}]}"#
    )
)]
pub fn timeline_tasks(ctx: Context, args: TimelineTasksArgs) -> Result<TimelineTasks> {
    let lists: Vec<Checklist> = storage::list_checklists()?
        .into_iter()
        .filter(|list| list.audience == lists::HOST)
        .collect();
    if lists.is_empty() {
        return Ok(TimelineTasks::default());
    }
    let items = storage::list_items()?;
    let states = storage::task_states(None)?;
    let mut tasks = Vec::new();
    for list in &lists {
        let list_items: Vec<&ChecklistItem> = items
            .iter()
            .filter(|item| item.checklist_id == list.id)
            .collect();
        if list_items.is_empty() {
            continue;
        }
        for plan in plan_tasks(list, &args.stays, &ctx.property.timezone) {
            if plan.at < args.from || plan.at > args.to {
                continue;
            }
            tasks.push(build_task(
                args.property_id,
                list,
                &list_items,
                &states,
                plan,
            ));
        }
    }
    tasks.sort_by_key(|task| task.at);
    Ok(TimelineTasks { tasks })
}

/// Where a task of `list` sits for one stay, before its items are attached.
#[derive(Debug, PartialEq)]
pub struct TaskPlan {
    pub stay_id: Uuid,
    pub at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub context: I18nText,
}

/// One plan per stay the list's trigger applies to.
pub fn plan_tasks(list: &Checklist, stays: &[TimelineStay], timezone: &str) -> Vec<TaskPlan> {
    let live: Vec<&TimelineStay> = stays.iter().filter(|stay| !is_cancelled(stay)).collect();
    let mut plans = Vec::new();
    for (index, stay) in live.iter().enumerate() {
        let next = live[index + 1..]
            .iter()
            .find(|next| next.check_in >= stay.check_out);
        let arrival_trigger = list.trigger == lists::BEFORE_EACH_ARRIVAL;
        if list.trigger == lists::ONLY_IF_NEXT_ARRIVAL && next.is_none() {
            continue;
        }
        // The arrival a deadline refers to: this stay's for an arrival task, else the next one.
        let arrival = if arrival_trigger {
            Some(stay.check_in)
        } else {
            next.map(|next| next.check_in)
        };
        let due_at = match list.deadline.as_deref() {
            Some(lists::ARRIVAL | lists::NEXT_ARRIVAL) => arrival,
            Some(lists::NEXT_ARRIVAL_MINUS_2H) => arrival.map(|at| at - Duration::hours(2)),
            Some(lists::DEPARTURE_EVENING) => Some(evening_of(stay.check_out, timezone)),
            _ => None,
        };
        let context = match (arrival_trigger, next) {
            (true, _) => i18n::text("task.context.beforeArrival", &[("guest", &stay.guest_name)]),
            (false, Some(next)) if list.trigger != lists::AT_DEPARTURE_BEFORE_CLEANING => {
                i18n::text("task.context.beforeArrival", &[("guest", &next.guest_name)])
            }
            _ => i18n::text(
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

fn build_task(
    property_id: Uuid,
    list: &Checklist,
    items: &[&ChecklistItem],
    states: &[TaskItemState],
    plan: TaskPlan,
) -> TimelineTask {
    let task_id = task_id(list.id, plan.stay_id);
    let mut task = timeline::task(
        task_id.clone(),
        plan.at,
        property_id,
        I18nText::new(list.name_fr.clone(), list.name_en.clone()),
        plan.context,
    )
    .stay(plan.stay_id)
    .items(
        items
            .iter()
            .map(|item| task_item(item, state_of(states, &task_id, item.id)))
            .collect(),
    );
    if let Some(due_at) = plan.due_at {
        task = task.due_at(due_at);
    }
    if let Some(name) = list.assignee_name.as_deref() {
        let role = list.assignee_role.clone().unwrap_or_default();
        task = task.assignee(name, I18nText::new(role.clone(), role));
    }
    task
}

fn state_of<'a>(
    states: &'a [TaskItemState],
    task_id: &str,
    item_id: Uuid,
) -> Option<&'a TaskItemState> {
    states
        .iter()
        .find(|state| state.task_id == task_id && state.item_id == item_id)
}

fn task_item(item: &ChecklistItem, state: Option<&TaskItemState>) -> TimelineTaskItem {
    let mut out = TimelineTaskItem::new(item.id.to_string(), labels::i18n_label(item));
    out.photo_required = item.photo_required;
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

#[portaki_sdk::command(name = "taskToggle")]
pub fn task_toggle(ctx: Context, args: TaskToggleArgs) -> Result<()> {
    let (list, items, stay_id) = load_task(&ctx, &args.task_id)?;
    let item = Uuid::parse_str(&args.item_id)
        .ok()
        .and_then(|id| items.iter().find(|item| item.id == id))
        .ok_or_else(|| PortakiError::Host("item_not_found".to_string()))?;
    let photo = match args
        .photo
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        None => None,
        Some(raw) => Some(
            FileRef::parse(raw)
                .ok_or_else(|| PortakiError::Host("invalid_photo".to_string()))?
                .to_string(),
        ),
    };
    let states = storage::task_states(Some(&args.task_id))?;
    let stored = state_of(&states, &args.task_id, item.id).and_then(|s| s.photo.clone());
    task_item(item, None).check_toggle(args.done, photo.as_deref().or(stored.as_deref()))?;

    storage::set_task_item(&args.task_id, item.id, args.done, photo, time::now()?)?;
    let restocked = args.done && is_restock(item);
    after_change(&ctx, &list, &items, &args.task_id, stay_id, restocked)
}

#[portaki_sdk::command(name = "taskComplete")]
pub fn task_complete(ctx: Context, args: TaskCompleteArgs) -> Result<()> {
    let (list, items, stay_id) = load_task(&ctx, &args.task_id)?;
    let states = storage::task_states(Some(&args.task_id))?;
    let open: Vec<&ChecklistItem> = items
        .iter()
        .filter(|item| !state_of(&states, &args.task_id, item.id).is_some_and(|s| s.done))
        .collect();
    // Refuse before ticking anything: a half-completed task would be worse than none.
    for item in &open {
        let stored = state_of(&states, &args.task_id, item.id).and_then(|s| s.photo.as_deref());
        task_item(item, None).check_toggle(true, stored)?;
    }
    let now = time::now()?;
    for item in &open {
        storage::set_task_item(&args.task_id, item.id, true, None, now)?;
    }
    let restocked = open.iter().any(|item| is_restock(item));
    after_change(&ctx, &list, &items, &args.task_id, stay_id, restocked)
}

fn load_task(ctx: &Context, task_id: &str) -> Result<(Checklist, Vec<ChecklistItem>, Uuid)> {
    if ctx.guest.is_some() {
        return Err(PortakiError::Host("host_only".to_string()));
    }
    let not_found = || PortakiError::Host("task_not_found".to_string());
    let (list_id, stay_id) = parse_task_id(task_id).ok_or_else(not_found)?;
    let list = storage::list_checklists()?
        .into_iter()
        .find(|list| list.id == list_id && list.audience == lists::HOST)
        .ok_or_else(not_found)?;
    let items = storage::items_of(list.id)?;
    Ok((list, items, stay_id))
}

fn is_restock(item: &ChecklistItem) -> bool {
    let label = labels::i18n_label(item);
    lists::is_restock_label(&label.fr, &label.en)
}

fn after_change(
    ctx: &Context,
    list: &Checklist,
    items: &[ChecklistItem],
    task_id: &str,
    stay_id: Uuid,
    restocked: bool,
) -> Result<()> {
    let states = storage::task_states(Some(task_id))?;
    let done = items
        .iter()
        .filter(|item| state_of(&states, task_id, item.id).is_some_and(|s| s.done))
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
            assignee_name: list.assignee_name.clone(),
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
                { "label": list.name_fr, "value": progress },
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

    fn list(trigger: &str, deadline: Option<&str>) -> Checklist {
        Checklist {
            id: Uuid::new_v4(),
            name_fr: "Ménage".into(),
            name_en: "Cleaning".into(),
            audience: lists::HOST.into(),
            icon: portaki_sdk::vocab::IconName::Sparkles.to_string(),
            trigger: trigger.into(),
            placement: lists::BOOKLET.into(),
            assignee_name: None,
            assignee_role: None,
            deadline: deadline.map(str::to_string),
            notify_assignee: false,
            alert_host: false,
            sort_order: 0,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
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
