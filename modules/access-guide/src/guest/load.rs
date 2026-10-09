//! Load config for guest surfaces.

use chrono::Timelike;
use portaki_sdk::host::time;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::GeoPoint;
use portaki_sdk::sdui::hours;

use crate::config::{has_content, HostConfig, ModuleConfig, UnlockWindow};
use crate::reveal::{evaluate_reveal, format_available_from, locked_message, RevealDecision};
use crate::texts::ModuleTexts;

pub struct GuestData {
    pub config: ModuleConfig,
    pub texts: ModuleTexts,
    pub address: String,
    /// The property on a map; `None` while it is not geocoded — no map then.
    pub coordinates: Option<GeoPoint>,
    /// Le nom du logement, pour nommer son repère sur le plan de la carte d'accès.
    pub property_name: String,
    pub secrets_revealed: bool,
    /// Preformatted guest message when secrets are locked (dated when possible).
    pub reveal_locked_message: Option<String>,
    /// The stay is over: codes are gone for good, and the status cell says so rather than
    /// falling back to the same wording as a code not yet due.
    pub reveal_ended: bool,
    /// When secrets open, written for the guest. `None` once revealed, once ended, or with no
    /// check-in to count from. The status cell puts it under the masked value.
    pub reveal_at_label: Option<String>,
    pub stay_id: Option<Uuid>,
    /// L'heure d'arrivée, à l'heure du logement — « 16:00 ». `None` sans séjour ou sans fuseau
    /// lisible : une heure dans le mauvais fuseau fait sonner à la porte trop tôt.
    pub checkin_hour: Option<String>,
    /// La consigne d'arrivée tardive de l'hôte, quand le voyageur en a annoncé une.
    ///
    /// `None` quand l'hôte n'a rien écrit — son silence ne se montre pas (§0.5) — ou quand
    /// l'arrivée annoncée n'est pas tardive : la consigne ne concerne alors personne.
    pub late_arrival_note: Option<String>,
    /// Le bouton « Déverrouiller » se montre (§2.4) : dans le séjour, ou comme les codes.
    pub unlock_open: bool,
    /// La réception aujourd'hui (§2.6) : ouverte à cette heure-ci, et ses plages du jour.
    /// `None` sans horaires en plages.
    pub desk_today: Option<DeskToday>,
}

/// La réception aujourd'hui, à l'heure du logement.
pub struct DeskToday {
    pub open_now: bool,
    /// « 07:00 – 12:00, 14:00 – 22:00 » ; vide un jour de fermeture.
    pub ranges: String,
}

pub enum GuestLoad {
    Ready(Box<GuestData>),
    /// Nothing written by the host yet.
    Empty,
}

pub fn load_guest_data(ctx: &GuestContext) -> Result<GuestLoad> {
    let host_config = HostConfig::load(ctx)?;
    let config = host_config.to_model(&ctx.locale);
    let texts = host_config.texts(&ctx.locale);
    if !has_content(&config, &texts) {
        return Ok(GuestLoad::Empty);
    }

    let configured_address = config.address().trim();
    let address = if configured_address.is_empty() {
        ctx.property.address.clone().unwrap_or_default()
    } else {
        configured_address.to_string()
    };

    let property_timezone = property_timezone(ctx);
    let checkin_at = ctx.stay.as_ref().and_then(|s| s.checkin_at);
    let checkout_at = ctx.stay.as_ref().and_then(|s| s.checkout_at);
    let stay_id = ctx.stay.as_ref().map(|s| s.stay_id);
    let now = time::now()?;
    let decision = evaluate_reveal(
        config.reveal_policy,
        config.reveal_hours,
        now,
        checkin_at,
        checkout_at,
        &property_timezone,
    );

    let late_arrival_note = late_arrival_note(ctx, &texts, &property_timezone);
    let desk_today = desk_today(&config.desk_hours, now, &property_timezone);
    let unlock_open = unlock_open(config.unlock_window, &decision, now, checkin_at);

    Ok(GuestLoad::Ready(Box::new(GuestData {
        config,
        texts,
        address,
        coordinates: ctx.property.coordinates,
        property_name: ctx.property.name.trim().to_string(),
        secrets_revealed: decision.revealed,
        reveal_locked_message: locked_banner(&decision, &property_timezone),
        reveal_ended: decision.ended,
        reveal_at_label: reveal_at_label(&decision, &property_timezone),
        stay_id,
        checkin_hour: checkin_hour(ctx, &property_timezone),
        late_arrival_note,
        unlock_open,
        desk_today,
    })))
}

/// De l'heure d'arrivée à l'heure de départ, par défaut ; comme les codes sinon. Sans heure
/// d'arrivée (un aperçu, un séjour sans dates), la fenêtre est celle des codes : ne rien montrer
/// cacherait le bouton à tort, l'ouvrir toujours le montrerait à qui n'a pas de séjour.
pub(crate) fn unlock_open(
    window: UnlockWindow,
    decision: &RevealDecision,
    now: chrono::DateTime<chrono::Utc>,
    checkin_at: Option<chrono::DateTime<chrono::Utc>>,
) -> bool {
    match (window, checkin_at) {
        (UnlockWindow::Stay, Some(checkin)) => now >= checkin && !decision.ended,
        _ => decision.revealed,
    }
}

fn property_timezone(ctx: &GuestContext) -> String {
    let from_property = ctx.property.timezone.trim();
    if !from_property.is_empty() {
        return from_property.to_string();
    }
    let from_ctx = ctx.timezone.trim();
    if !from_ctx.is_empty() {
        return from_ctx.to_string();
    }
    "Europe/Paris".to_string()
}

fn locked_banner(decision: &RevealDecision, property_timezone: &str) -> Option<String> {
    if decision.revealed || decision.ended {
        return None;
    }
    let when = decision
        .available_from
        .map(|at| format_available_from(at, property_timezone));
    Some(locked_message(when.as_deref()))
}

/// The reveal instant alone, without the sentence around it.
///
/// The banner reads « Disponible à partir du 23/08/2026 à 16:00 », which is too long for a cell
/// of the status strip: the cell shows the date on its own line under the masked value.
fn reveal_at_label(decision: &RevealDecision, property_timezone: &str) -> Option<String> {
    if decision.revealed || decision.ended {
        return None;
    }
    decision
        .available_from
        .map(|at| format_available_from(at, property_timezone))
}

/// « 16:00 », à l'heure du logement.
/// Après cette heure, une arrivée est tardive où que soit le logement (§2.1).
const LATE_HOUR: u32 = 22;

/// Les heures après l'entrée au bout desquelles une arrivée est tardive.
///
/// C'est la borne du dernier créneau que `pre-arrival-form` propose — « après 19 h » pour une
/// entrée à 16 h. Sans elle, la consigne serait morte dans presque tous les logements : l'heure
/// annoncée est le **début** du créneau choisi, donc elle n'atteint jamais 22 h quand l'entrée
/// est à 16 h.
const LATE_AFTER_CHECKIN_HOURS: u32 = 3;

/// La consigne d'arrivée tardive, quand elle a un destinataire.
///
/// Trois conditions, et le silence de l'une suffit : l'hôte a écrit quelque chose, le voyageur a
/// annoncé une heure en pré-arrivée (la plateforme la recopie sur le séjour), et cette heure est
/// tardive.
fn late_arrival_note(
    ctx: &GuestContext,
    texts: &ModuleTexts,
    property_timezone: &str,
) -> Option<String> {
    let note = texts.late_arrival_note.trim();
    if note.is_empty() {
        return None;
    }
    let stay = ctx.stay.as_ref()?;
    let announced = stay.arrival_time_estimated?.hour();
    let after_checkin = checkin_local_hour(ctx, property_timezone)
        .map(|hour| hour + LATE_AFTER_CHECKIN_HOURS)
        // Une entrée du soir fait déborder la borne sur le lendemain : seule l'heure fixe vaut.
        .filter(|hour| *hour <= 23);
    let late = announced >= LATE_HOUR || after_checkin.is_some_and(|hour| announced >= hour);
    late.then(|| note.to_string())
}

/// « Ouvert maintenant » (§2.6) : l'heure du logement dans une plage du jour. Une plage qui
/// passe minuit court jusqu'à la fin du jour, comme dans `hours::overlaps`.
pub(crate) fn desk_today(
    week: &str,
    now: chrono::DateTime<chrono::Utc>,
    property_timezone: &str,
) -> Option<DeskToday> {
    use chrono::Datelike;
    let week = hours::parse_week(week);
    if week.iter().all(Vec::is_empty) {
        return None;
    }
    let local = portaki_sdk::host::time::PropertyTz::parse(property_timezone)?.to_local(now);
    let today = &week[local.weekday().num_days_from_monday() as usize];
    let at = local.format("%H:%M").to_string();
    Some(DeskToday {
        open_now: today
            .iter()
            .any(|(start, end)| *start <= at && (end < start || at < *end)),
        ranges: format_ranges(today),
    })
}

/// « 07:00 – 12:00, 14:00 – 22:00 ».
pub(crate) fn format_ranges(ranges: &[hours::Range]) -> String {
    ranges
        .iter()
        .map(|(start, end)| format!("{start} – {end}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn checkin_local_hour(ctx: &GuestContext, property_timezone: &str) -> Option<u32> {
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at)?;
    let tz = portaki_sdk::host::time::PropertyTz::parse(property_timezone)?;
    Some(tz.to_local(checkin_at).hour())
}

fn checkin_hour(ctx: &GuestContext, property_timezone: &str) -> Option<String> {
    let checkin_at = ctx.stay.as_ref().and_then(|stay| stay.checkin_at)?;
    let tz = portaki_sdk::host::time::PropertyTz::parse(property_timezone)?;
    Some(tz.to_local(checkin_at).format("%H:%M").to_string())
}
