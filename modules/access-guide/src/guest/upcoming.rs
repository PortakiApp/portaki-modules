//! Compact pre-arrival prep card for the upcoming timeline.
//!
//! Shows a single glanceable line — the primary access method — instead of the
//! full access glance (map, codes, steps) rendered by the home card.

use portaki_sdk::prelude::*;

use portaki_sdk::sdui::primitives::{Card, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{DoorCodeTarget, MethodFields, StaffKind};

use super::load::GuestData;

pub fn build_upcoming_card(data: &GuestData) -> Surface {
    Surface::new(
        Card::new()
            .icon("car")
            .title("i18n:nav.access-guide")
            .child(
                Text::new()
                    .text(method_label_key(data))
                    .variant(TextVariant::Body),
            ),
    )
    .with_id(crate::ids::UPCOMING_CARD)
}

/// One key live value: the primary access method label (i18n reference).
fn method_label_key(data: &GuestData) -> &'static str {
    match &data.config.method {
        MethodFields::Keybox { .. } => "i18n:guest.method.keybox",
        MethodFields::DoorCode { target, .. } => match target {
            DoorCodeTarget::Gate => "i18n:guest.doorCode.gate",
            DoorCodeTarget::Building => "i18n:guest.doorCode.building",
            DoorCodeTarget::Apartment => "i18n:guest.doorCode.apartment",
        },
        MethodFields::SmartLock { .. } => "i18n:guest.method.smartLock",
        MethodFields::InPerson { .. } => "i18n:guest.method.inPerson",
        MethodFields::BuildingStaff { staff_kind, .. } => match staff_kind {
            StaffKind::Reception => "i18n:guest.buildingStaff.reception",
            StaffKind::Caretaker => "i18n:guest.buildingStaff.caretaker",
        },
        MethodFields::HostGreets { .. } => "i18n:guest.method.hostGreets",
        MethodFields::Other {} => "i18n:guest.method.other",
    }
}
