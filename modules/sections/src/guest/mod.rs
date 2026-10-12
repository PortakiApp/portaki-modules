//! Guest booklet surfaces. The SDK renders the inactive / incomplete / error states.

mod home;
mod sheet;

use chrono::{DateTime, Days, NaiveTime, Utc};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::EmptyState;
use portaki_sdk::sdui::surface::Surface;

use home::build_home_card;
use sheet::build_sheet_surface;

use crate::config::{ModuleConfig, ShowFrom};
use crate::model::SectionView;
use crate::queries::{list_sections, ListSectionsArgs};

#[portaki_sdk::surface(guest, id = "home.card")]
pub fn render_home_card(ctx: GuestContext) -> Result<Surface> {
    // La signature de l'hôte vient de la plateforme : « Claire, votre hôte depuis 2019 ». Un
    // module qui redemanderait son nom en aurait un différent dans chaque module.
    let byline = host_byline(&ctx);
    render_with_sections(&ctx, HOME_CARD, move |sections| {
        build_home_card(sections, &byline)
    })
}

/// « {prénom}, {rôle} » — vide quand la plateforme ne sert pas d'hôte (un aperçu, un espace sans
/// propriétaire). Mieux vaut pas de sous-titre qu'une virgule orpheline.
fn host_byline(ctx: &GuestContext) -> String {
    let Some(host) = ctx.host.as_ref() else {
        return String::new();
    };
    let name = host.name.trim();
    match host
        .role
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty())
    {
        Some(role) if !name.is_empty() => format!("{name}, {role}"),
        Some(role) => role.to_string(),
        None => name.to_string(),
    }
}

#[portaki_sdk::surface(guest, id = "explore.sheet")]
pub fn render_explore_sheet(ctx: GuestContext) -> Result<Surface> {
    render_with_sections(&ctx, EXPLORE_SHEET, build_sheet_surface)
}

fn render_with_sections(
    ctx: &GuestContext,
    surface_id: SurfaceId,
    build: impl FnOnce(&[SectionView]) -> Surface,
) -> Result<Surface> {
    let config = ModuleConfig::load(ctx)?;
    if !is_shown(config.show_from, ctx)? {
        return Ok(no_sections_state(surface_id));
    }
    let sections = list_sections(
        ctx.clone(),
        ListSectionsArgs {
            locale: Some(ctx.locale.clone()),
        },
    )?;
    let visible: Vec<SectionView> = sections.into_iter().filter(|s| !s.is_blank()).collect();
    if visible.is_empty() {
        return Ok(no_sections_state(surface_id));
    }
    Ok(build(&visible))
}

/// « Afficher à partir de » (§2.1), against the stay of this invocation.
fn is_shown(show_from: ShowFrom, ctx: &GuestContext) -> Result<bool> {
    let checkin = ctx.stay.as_ref().and_then(|stay| stay.checkin_at);
    if show_from == ShowFrom::Booking || checkin.is_none() {
        return Ok(true);
    }
    Ok(opens_by(
        show_from,
        time::now()?,
        checkin,
        &ctx.property.timezone,
    ))
}

/// Days are counted in the property's timezone: « La veille » opens at midnight the day before
/// check-in, « Le jour » at midnight of the check-in day. A stay booked the day before with « La
/// veille » is therefore shown at once (§9 cas 5). No check-in known — a preview, the public page
/// — fails open: the word of welcome is never hidden for want of a date.
pub fn opens_by(
    show_from: ShowFrom,
    now: DateTime<Utc>,
    checkin: Option<DateTime<Utc>>,
    timezone: &str,
) -> bool {
    let days_before = match show_from {
        ShowFrom::Booking => return true,
        ShowFrom::DayBefore => 1,
        ShowFrom::ArrivalDay => 0,
    };
    let Some(checkin) = checkin else {
        return true;
    };
    // ponytail: an unknown zone counts in UTC, off by its offset at most.
    let tz = PropertyTz::parse(timezone);
    let local_day = tz
        .as_ref()
        .map_or(checkin.date_naive(), |tz| tz.to_local(checkin).date_naive());
    let Some(day) = local_day.checked_sub_days(Days::new(days_before)) else {
        return true;
    };
    let midnight = day.and_time(NaiveTime::MIN);
    let opens = tz
        .as_ref()
        .map_or(midnight.and_utc(), |tz| tz.from_local(midnight));
    now >= opens
}

/// No section written yet, or not shown yet.
fn no_sections_state(surface_id: SurfaceId) -> Surface {
    Surface::new(
        EmptyState::new()
            .title("i18n:home.card.empty.title")
            .description("i18n:home.card.empty.description")
            .icon(IconName::Home),
    )
    .with_id(surface_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .unwrap()
            .with_timezone(&Utc)
    }

    const PARIS: &str = "Europe/Paris";

    #[test]
    fn the_eve_opens_at_local_midnight_the_day_before() {
        let checkin = Some(utc("2026-07-20T14:00:00Z"));
        // Minuit à Paris (UTC+2) : 22 h UTC la veille de la veille.
        assert!(!opens_by(
            ShowFrom::DayBefore,
            utc("2026-07-18T21:59:00Z"),
            checkin,
            PARIS
        ));
        assert!(opens_by(
            ShowFrom::DayBefore,
            utc("2026-07-18T22:00:00Z"),
            checkin,
            PARIS
        ));
    }

    #[test]
    fn the_day_opens_at_local_midnight_of_check_in() {
        let checkin = Some(utc("2026-07-20T14:00:00Z"));
        assert!(!opens_by(
            ShowFrom::ArrivalDay,
            utc("2026-07-19T21:59:00Z"),
            checkin,
            PARIS
        ));
        assert!(opens_by(
            ShowFrom::ArrivalDay,
            utc("2026-07-19T22:00:00Z"),
            checkin,
            PARIS
        ));
    }

    #[test]
    fn booking_and_unknown_dates_always_show() {
        let far = Some(utc("2027-01-01T14:00:00Z"));
        let now = utc("2026-07-01T10:00:00Z");
        assert!(opens_by(ShowFrom::Booking, now, far, PARIS));
        assert!(opens_by(ShowFrom::DayBefore, now, None, PARIS));
        assert!(!opens_by(ShowFrom::DayBefore, now, far, PARIS));
    }

    /// §9 cas 5 : réservé la veille, « La veille » → visible tout de suite.
    #[test]
    fn a_stay_booked_the_day_before_shows_at_once() {
        let checkin = Some(utc("2026-07-20T14:00:00Z"));
        assert!(opens_by(
            ShowFrom::DayBefore,
            utc("2026-07-19T09:00:00Z"),
            checkin,
            PARIS
        ));
    }
}
