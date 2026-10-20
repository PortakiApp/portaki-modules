//! Checklist wire values.

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
