//! The access cell of the booklet status strip (§1.5).
//!
//! The strip above the sections says where the stay stands: how far the arrival is, which night
//! it is, when checkout falls. One cell of it is not the booklet's to write — what the guest
//! opens the door with. That cell belongs to whoever knows the door, and the booklet draws it
//! without knowing which module filled it.
//!
//! So the module sends one [`KeyValue`]: a label, a value, and whether the value is still a
//! secret. It sends no colour, no position and no size — the strip decides those (§0.2).
//!
//! The wording follows the access method, because « Code » is wrong for a key handover and
//! « Rendez-vous » is wrong for a keybox:
//!
//! | Method | Label | Value |
//! |---|---|---|
//! | Keybox | Code boîte à clés | the code |
//! | Door code | Code portail / immeuble / appartement | the code |
//! | Smart lock | Code de secours | the backup code |
//! | In person | Remise des clés | the agreed slot |
//! | Building staff | Clés | Réception / Gardien |
//! | Host greets | Accueil | the host |
//! | Other | Accès | Consignes |
//!
//! Before the reveal the label says « Infos d'accès » and nothing else: a cell that said
//! « Code boîte à clés · •••••• » would tell a guest reading over a shoulder that there is a
//! keybox, and the booklet is careful never to name the way in before its time.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::{KeyValueLayout, SecretState};
use portaki_sdk::sdui::primitives::KeyValue;
use portaki_sdk::sdui::surface::Surface;

use crate::config::{DoorCodeTarget, MethodFields, StaffKind};
use crate::reveal::SECRET_MASK;

use super::load::GuestData;

/// What the cell shows: a label, a value, and how to read the value.
struct Cell {
    label: &'static str,
    value: String,
    /// The value is a code or a time: it reads in aligned figures.
    mono: bool,
    icon: IconName,
}

pub fn build_status_cell(data: &GuestData) -> Surface {
    Surface::new(cell_component(data)).with_id(crate::guest::STATUS_CELL)
}

fn cell_component(data: &GuestData) -> KeyValue {
    // After checkout the codes are gone and never come back (§1.10). The cell says the access is
    // over rather than reusing the wording of a code not yet due.
    if data.reveal_ended {
        return tile(Cell {
            label: "i18n:guest.status.access",
            value: "i18n:guest.status.access.ended".into(),
            mono: false,
            icon: IconName::Lock,
        });
    }

    // A method that carries no code is not a secret: a meeting slot or a reception desk shows
    // straight away, and the reveal schedule has nothing to hold back.
    let Some(secret) = secret_cell(data) else {
        return tile(open_cell(data));
    };

    if data.secrets_revealed {
        return tile(secret).copy(true);
    }

    // Masked: the label gives away neither the method nor the kind of secret.
    tile(Cell {
        label: "i18n:guest.status.pending",
        value: SECRET_MASK.into(),
        mono: true,
        icon: IconName::Lock,
    })
    .secret(SecretState::hidden(data.reveal_at_label.clone()))
}

fn tile(cell: Cell) -> KeyValue {
    let mut node = KeyValue::new()
        .key(cell.label)
        .value(cell.value)
        .layout(KeyValueLayout::Tile)
        .icon(cell.icon);
    if cell.mono {
        node = node.mono(true);
    }
    node
}

/// The cell for a method whose value is a secret, when it has one to show.
///
/// `None` when the method carries no code at all, or when the host picked a code-bearing method
/// and left the code empty — then the cell falls back to naming the method, which is true and
/// useless rather than false.
fn secret_cell(data: &GuestData) -> Option<Cell> {
    let (label, code) = match &data.config.method {
        MethodFields::Keybox { code, .. } => ("i18n:guest.status.keybox", code.as_deref()),
        MethodFields::DoorCode { target, code } => (door_code_label(*target), Some(code.as_str())),
        MethodFields::SmartLock { manual_code } => {
            ("i18n:guest.status.smartLock", manual_code.as_deref())
        }
        MethodFields::InPerson { .. }
        | MethodFields::BuildingStaff { .. }
        | MethodFields::HostGreets { .. }
        | MethodFields::Other {} => return None,
    };
    let code = code.map(str::trim).filter(|code| !code.is_empty())?;
    Some(Cell {
        label,
        value: code.to_string(),
        mono: true,
        icon: method_icon(&data.config.method),
    })
}

/// The cell for a method the guest may read at any time.
fn open_cell(data: &GuestData) -> Cell {
    match &data.config.method {
        MethodFields::InPerson {
            meeting_place,
            time_hint,
            ..
        } => Cell {
            label: "i18n:guest.status.inPerson",
            // The slot is what the guest needs at a glance; the place is in the Access page.
            value: first_filled(&[time_hint.as_deref(), Some(meeting_place)])
                .unwrap_or_else(|| "i18n:guest.status.inPerson.toAgree".into()),
            mono: false,
            icon: method_icon(&data.config.method),
        },
        MethodFields::BuildingStaff { staff_kind, .. } => Cell {
            label: "i18n:guest.status.keys",
            value: staff_label(*staff_kind).into(),
            mono: false,
            icon: method_icon(&data.config.method),
        },
        MethodFields::HostGreets { eta_hint, .. } => Cell {
            label: "i18n:guest.status.welcome",
            // §1.5 asks for the host's name here. `Context.host` exists in the SDK and the
            // runtime does not fill it yet, so the cell names the role instead of a stranger.
            value: first_filled(&[eta_hint.as_deref()])
                .unwrap_or_else(|| "i18n:guest.status.welcome.host".into()),
            mono: false,
            icon: method_icon(&data.config.method),
        },
        // A code-bearing method with no code written, or `Other`: the cell points at the page
        // rather than claiming a value it does not have.
        MethodFields::Keybox { .. }
        | MethodFields::DoorCode { .. }
        | MethodFields::SmartLock { .. }
        | MethodFields::Other {} => Cell {
            label: "i18n:guest.status.access",
            value: "i18n:guest.status.access.instructions".into(),
            mono: false,
            icon: method_icon(&data.config.method),
        },
    }
}

/// The icon that names the way in, for any surface that announces the method.
///
/// One mapping, because the strip cell and the pre-arrival card name the same thing: a keybox is
/// a key, a meeting is an hour, a reception desk is a handover. The `Lock` of a masked cell is
/// not here — that one follows the reveal schedule, not the method.
pub(super) fn method_icon(method: &MethodFields) -> IconName {
    match method {
        // A code to type, whatever it opens — and `Other`, where the page holds the instructions.
        MethodFields::Keybox { .. }
        | MethodFields::DoorCode { .. }
        | MethodFields::SmartLock { .. }
        | MethodFields::Other {} => IconName::Key,
        MethodFields::InPerson { .. } => IconName::Clock,
        MethodFields::BuildingStaff { .. } => IconName::Handshake,
        MethodFields::HostGreets { .. } => IconName::User,
    }
}

fn first_filled(candidates: &[Option<&str>]) -> Option<String> {
    candidates
        .iter()
        .flatten()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

fn door_code_label(target: DoorCodeTarget) -> &'static str {
    match target {
        DoorCodeTarget::Gate => "i18n:guest.status.doorCode.gate",
        DoorCodeTarget::Building => "i18n:guest.status.doorCode.building",
        DoorCodeTarget::Apartment => "i18n:guest.status.doorCode.apartment",
    }
}

fn staff_label(kind: StaffKind) -> &'static str {
    match kind {
        StaffKind::Reception => "i18n:guest.buildingStaff.reception",
        StaffKind::Caretaker => "i18n:guest.buildingStaff.caretaker",
    }
}
