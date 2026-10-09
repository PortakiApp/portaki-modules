//! « Rappel » (spec Checklist §2.1): the guest is reminded of a departure list left unfinished.
//!
//! ponytail: the spec asks for a notification at 08:00 local on departure day. Modules have no
//! push channel and the platform runs declared e-mails once a day at 08:00 UTC — which is already
//! past a 10:00 CEST check-out — so this is an e-mail the eve of check-out. An hour-precise push
//! needs a platform channel first. A `departureDay` list is not open yet on the eve: no reminder.

use portaki_sdk::host::email::{
    self, EmailAudience, ModuleEmailCta, ModuleEmailSdui, SendEmailArgs,
};
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;

use crate::email_i18n;
use crate::lists;
use crate::show_when::is_checklist_available;
use crate::storage;

pub const DEPARTURE_REMINDER_EMAIL_ID: &str = "departure-reminder";

#[portaki_sdk::email(
    id = "departure-reminder",
    audience = EmailAudience::Guest,
    trigger = EmailTrigger::RelativeToCheckOut,
    offset_days = -1,
    requires_guest_email,
    description_key = "email.departure-reminder.description",
    skip_when = SkipWhen::GuestEmailMissing,
    skip_when = SkipWhen::StayCancelled
)]
#[portaki_sdk::command(
    name = "sendDepartureReminder",
    example(label = "Rappel la veille du départ")
)]
pub fn send_departure_reminder(ctx: Context, _args: EmptyArgs) -> Result<()> {
    let stay_id = ctx
        .guest
        .as_ref()
        .map(|guest| guest.session_id)
        .or_else(|| ctx.stay.as_ref().map(|stay| stay.stay_id))
        .ok_or_else(|| PortakiError::Host("stay_id_required".to_string()))?;
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    let checkout_at = ctx.stay.as_ref().and_then(|stay| stay.checkout_at);
    let now = time::now()?;
    // A stay shortened since the tick picked it: a guest already gone is not reminded.
    if checkout_at.is_some_and(|checkout| now >= checkout) {
        return Ok(());
    }
    let done: Vec<_> = storage::list_completions(Some(stay_id))?
        .into_iter()
        .map(|row| row.item_id)
        .collect();
    let mut unfinished = false;
    for list in storage::list_checklists()? {
        if list.audience != lists::GUEST
            || !storage::display::read(list.id).remind()
            || !is_checklist_available(
                &list.trigger,
                now,
                checkin_at,
                checkout_at,
                &ctx.property.timezone,
            )
        {
            continue;
        }
        if storage::items_of(list.id)?
            .iter()
            .any(|item| !done.contains(&item.id))
        {
            unfinished = true;
            break;
        }
    }
    if !unfinished {
        return Ok(());
    }

    email::send(&SendEmailArgs {
        email_id: DEPARTURE_REMINDER_EMAIL_ID.into(),
        audience: EmailAudience::Guest,
        content: ModuleEmailSdui {
            subject: email_i18n::text("email.departureReminder.subject"),
            eyebrow: Some(email_i18n::text("email.departureReminder.eyebrow")),
            title: Some(email_i18n::text("email.departureReminder.title")),
            body: email_i18n::text("email.departureReminder.body"),
            cta: Some(ModuleEmailCta {
                label: email_i18n::text("email.departureReminder.cta"),
                url: None,
                portaki_action: Some("open-module:checklist:default".into()),
            }),
            ..Default::default()
        },
        stay_id: Some(stay_id),
        property_id: None,
        action_url: None,
    })
}
