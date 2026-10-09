//! Report status wire values (spec Objet oublié §1, `property-workspace-tab`).

use portaki_sdk::prelude::*;

/// Allowed status values on the wire, in the spec's order.
///
/// | Wire        | FR (UI)     |
/// |-------------|-------------|
/// | `declared`  | Déclaré     |
/// | `found`     | Trouvé      |
/// | `shipped`   | Renvoyé     |
/// | `picked_up` | Retiré      |
/// | `donated`   | Donné       |
/// | `not_found` | Introuvable |
pub const WIRE_VALUES: &[&str] = &[
    "declared",
    "found",
    "shipped",
    "picked_up",
    "donated",
    "not_found",
];

/// A guest's declaration starts « Déclaré ».
pub const DEFAULT: &str = "declared";

/// An item the host declares starts « Trouvé » : he has it in hand.
pub const FOUND: &str = "found";

/// The item went back to its owner, or to the association.
pub const RETURNED: &[&str] = &["shipped", "picked_up", "donated"];

/// The stored value in the current set. Rows written before the six statuses keep their old
/// value in the database — `to_collect`, `sent`, `returned` — and read as the nearest new one:
/// « À récupérer » is « Déclaré » for a guest report, « Trouvé » for an item the host found.
pub fn normalize(raw: &str, kind: &str) -> &'static str {
    match raw.trim() {
        "to_collect" | "" if kind == "found" => FOUND,
        "to_collect" | "" => DEFAULT,
        "sent" => "shipped",
        "returned" => "picked_up",
        other => WIRE_VALUES
            .iter()
            .copied()
            .find(|wire| *wire == other)
            .unwrap_or(DEFAULT),
    }
}

/// Validates a status sent by the host. Legacy values are still accepted (a dashboard tab left
/// open across the release sends them) and read as their new equivalent.
pub fn parse_status(raw: &str) -> Result<&'static str> {
    let trimmed = raw.trim();
    match trimmed {
        "to_collect" => Ok(DEFAULT),
        "sent" => Ok("shipped"),
        "returned" => Ok("picked_up"),
        _ => WIRE_VALUES
            .iter()
            .copied()
            .find(|wire| *wire == trimmed)
            .ok_or_else(|| PortakiError::Host(format!("invalid_status:{trimmed}"))),
    }
}

/// Where an item can go from `from`. Returned items are final; an item not found can still turn
/// up. Declared goes straight to a return too: the host who finds and posts it the same day does
/// not have to click « Trouvé » first.
pub fn next(from: &str) -> &'static [&'static str] {
    match from {
        "declared" => &["found", "shipped", "picked_up", "donated", "not_found"],
        "found" => &["shipped", "picked_up", "donated"],
        "not_found" => &["found"],
        _ => &[],
    }
}

/// `to` is reachable from `from` — staying put is always allowed.
pub fn can_move(from: &str, to: &str) -> bool {
    from == to || next(from).contains(&to)
}

/// The item is with its owner or the association.
pub fn is_returned(wire: &str) -> bool {
    RETURNED.contains(&wire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_values_read_as_the_new_set() {
        assert_eq!(normalize("to_collect", "lost"), "declared");
        assert_eq!(normalize("to_collect", "found"), "found");
        assert_eq!(normalize("sent", "lost"), "shipped");
        assert_eq!(normalize("returned", "found"), "picked_up");
        assert_eq!(normalize("", "lost"), "declared");
        assert_eq!(normalize("garbage", "lost"), "declared");
        for wire in WIRE_VALUES {
            assert_eq!(normalize(wire, "lost"), *wire);
        }
    }

    #[test]
    fn parse_accepts_the_six_and_the_legacy_three() {
        for wire in WIRE_VALUES {
            assert_eq!(parse_status(wire).unwrap(), *wire);
        }
        assert_eq!(parse_status("sent").unwrap(), "shipped");
        assert_eq!(parse_status("returned").unwrap(), "picked_up");
        assert_eq!(parse_status("to_collect").unwrap(), "declared");
        assert!(parse_status("lost").is_err());
    }

    #[test]
    fn transitions() {
        assert!(can_move("declared", "found"));
        assert!(can_move("declared", "not_found"));
        assert!(can_move("declared", "shipped"));
        assert!(can_move("found", "picked_up"));
        assert!(can_move("found", "donated"));
        assert!(!can_move("found", "declared"));
        assert!(!can_move("found", "not_found"));
        assert!(can_move("not_found", "found"));
        assert!(!can_move("not_found", "shipped"));
        for done in RETURNED {
            assert!(can_move(done, done));
            assert!(next(done).is_empty());
        }
    }
}
