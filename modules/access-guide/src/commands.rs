//! Module commands — configuration persistence.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{
    config_from_update_parts, save_config, AccessStep, ArrivalGuide, BuildingAccess, DoorCodeTarget,
    Localized, MethodFields, ModuleConfig, ParkingLayer, PrimaryMethod, RevealPolicy, StaffKind,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepInput {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub title_fr: String,
    #[serde(default)]
    pub title_en: String,
    #[serde(default)]
    pub detail_fr: String,
    #[serde(default)]
    pub detail_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateConfigArgs {
    // ── New schema (structured) ──────────────────────────────────────────────
    #[serde(default)]
    pub primary_method: Option<PrimaryMethod>,
    #[serde(default)]
    pub method: Option<MethodFields>,
    #[serde(default)]
    pub building_access: Option<BuildingAccess>,
    #[serde(default)]
    pub parking: Option<ParkingLayer>,
    #[serde(default)]
    pub arrival: Option<ArrivalGuide>,
    #[serde(default)]
    pub reveal_policy: Option<RevealPolicy>,
    #[serde(default)]
    pub smart_lock_provider_module_id: Option<String>,

    // ── Host form flat fields (assembled when primary_method is set) ─────────
    #[serde(default)]
    pub building_access_enabled: Option<bool>,
    #[serde(default)]
    pub parking_enabled: Option<bool>,

    #[serde(default)]
    pub keybox_location: String,
    #[serde(default)]
    pub keybox_instructions: String,
    #[serde(default)]
    pub door_code_target: String,
    #[serde(default)]
    pub door_code: String,
    #[serde(default)]
    pub door_code_instructions: String,
    #[serde(default)]
    pub smart_lock_instructions: String,
    #[serde(default)]
    pub smart_lock_manual_code: String,
    #[serde(default)]
    pub smart_lock_provider_module_id_custom: String,
    #[serde(default)]
    pub in_person_meeting_place: String,
    #[serde(default)]
    pub in_person_meeting_lat: String,
    #[serde(default)]
    pub in_person_meeting_lng: String,
    #[serde(default)]
    pub in_person_time_hint: String,
    #[serde(default)]
    pub in_person_contact: String,
    #[serde(default)]
    pub building_staff_kind: String,
    #[serde(default)]
    pub building_staff_desk_location: String,
    #[serde(default)]
    pub building_staff_hours: String,
    #[serde(default)]
    pub building_staff_contact: String,
    #[serde(default)]
    pub host_greets_contact_note: String,
    #[serde(default)]
    pub host_greets_eta_hint: String,
    #[serde(default)]
    pub other_instructions: String,

    #[serde(default)]
    pub building_access_gate_code: String,
    #[serde(default)]
    pub building_access_intercom: String,
    #[serde(default)]
    pub building_access_note: String,
    #[serde(default)]
    pub parking_code: String,

    // ── Legacy / host-form flat fields (migrated on save) ────────────────────
    #[serde(default)]
    pub steps: Vec<StepInput>,
    #[serde(default)]
    pub steps_json: String,
    #[serde(default)]
    pub parking_map_url: String,
    #[serde(default)]
    pub arrival_video_url: String,
    #[serde(default)]
    pub global_note: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub gate_code: String,
    #[serde(default)]
    pub keybox_code: String,
    #[serde(default)]
    pub parking_info: String,
}

impl UpdateConfigArgs {
    fn resolve_steps(&self) -> Vec<AccessStep> {
        if !self.steps.is_empty() {
            return self
                .steps
                .iter()
                .enumerate()
                .filter_map(|(index, input)| step_from_input(input, index))
                .collect();
        }
        let raw = self.steps_json.trim();
        if raw.is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<AccessStep>>(raw)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !s.id.trim().is_empty())
            .collect()
    }

    fn into_config(self) -> ModuleConfig {
        // Structured `method` wins (API / tests).
        if self.method.is_some() {
            return self.into_structured_config();
        }
        // Host SDUI posts primary_method + flat fields.
        if let Some(primary) = self.primary_method {
            return self.assemble_host_form(primary);
        }
        // Legacy flat gate/keybox only.
        self.into_legacy_config()
    }

    fn into_structured_config(self) -> ModuleConfig {
        let steps = self.resolve_steps();
        config_from_update_parts(
            self.primary_method,
            self.method,
            self.building_access,
            self.parking,
            self.arrival,
            self.reveal_policy,
            self.smart_lock_provider_module_id,
            steps,
            self.steps_json,
            self.parking_map_url,
            self.arrival_video_url,
            self.global_note,
            self.address,
            self.gate_code,
            self.keybox_code,
            self.parking_info,
        )
    }

    fn into_legacy_config(self) -> ModuleConfig {
        let steps = self.resolve_steps();
        config_from_update_parts(
            None,
            None,
            None,
            None,
            None,
            self.reveal_policy,
            self.smart_lock_provider_module_id,
            steps,
            self.steps_json,
            self.parking_map_url,
            self.arrival_video_url,
            self.global_note,
            self.address,
            self.gate_code,
            self.keybox_code,
            self.parking_info,
        )
    }

    fn assemble_host_form(self, primary: PrimaryMethod) -> ModuleConfig {
        let method = assemble_method(&self, primary);
        let building_access = assemble_building_access(&self);
        let parking = assemble_parking(&self);
        let steps = self.resolve_steps();
        let arrival = ArrivalGuide {
            address: self.address.trim().to_string(),
            steps,
            arrival_video_url: self.arrival_video_url.trim().to_string(),
            global_note: self.global_note.trim().to_string(),
        };
        let provider = if primary == PrimaryMethod::SmartLock {
            resolve_smart_lock_provider(&self)
        } else {
            None
        };

        let mut config = ModuleConfig {
            primary_method: primary,
            method,
            building_access,
            parking,
            arrival,
            reveal_policy: self.reveal_policy.unwrap_or(RevealPolicy::DayBefore16h),
            smart_lock_provider_module_id: provider,
        };
        config.sync_primary_method();
        config
    }
}

fn assemble_method(args: &UpdateConfigArgs, primary: PrimaryMethod) -> MethodFields {
    match primary {
        PrimaryMethod::Keybox => MethodFields::Keybox {
            location: args.keybox_location.trim().to_string(),
            code: nonempty_owned(&args.keybox_code),
            instructions: nonempty_owned(&args.keybox_instructions),
        },
        PrimaryMethod::DoorCode => {
            let code = if !args.door_code.trim().is_empty() {
                args.door_code.trim().to_string()
            } else {
                args.gate_code.trim().to_string()
            };
            MethodFields::DoorCode {
                target: parse_door_target(&args.door_code_target),
                code,
                instructions: nonempty_owned(&args.door_code_instructions),
            }
        }
        PrimaryMethod::SmartLock => MethodFields::SmartLock {
            instructions: nonempty_owned(&args.smart_lock_instructions),
            manual_code: nonempty_owned(&args.smart_lock_manual_code),
        },
        PrimaryMethod::InPerson => {
            let (lat, lng) =
                parse_optional_coord_pair(&args.in_person_meeting_lat, &args.in_person_meeting_lng);
            MethodFields::InPerson {
                meeting_place: args.in_person_meeting_place.trim().to_string(),
                lat,
                lng,
                time_hint: nonempty_owned(&args.in_person_time_hint),
                contact: nonempty_owned(&args.in_person_contact),
            }
        }
        PrimaryMethod::BuildingStaff => MethodFields::BuildingStaff {
            staff_kind: parse_staff_kind(&args.building_staff_kind),
            desk_location: args.building_staff_desk_location.trim().to_string(),
            hours: nonempty_owned(&args.building_staff_hours),
            contact: nonempty_owned(&args.building_staff_contact),
        },
        PrimaryMethod::HostGreets => MethodFields::HostGreets {
            contact_note: nonempty_owned(&args.host_greets_contact_note),
            eta_hint: nonempty_owned(&args.host_greets_eta_hint),
        },
        PrimaryMethod::Other => MethodFields::Other {
            instructions: args.other_instructions.trim().to_string(),
        },
    }
}

fn assemble_building_access(args: &UpdateConfigArgs) -> Option<BuildingAccess> {
    if args.building_access_enabled == Some(false) {
        return None;
    }
    if let Some(structured) = args.building_access.clone() {
        if args.building_access_enabled != Some(true) && !has_building_flat(args) {
            return if structured.is_empty() {
                None
            } else {
                Some(structured)
            };
        }
    }
    if args.building_access_enabled != Some(true) && !has_building_flat(args) {
        return None;
    }
    let layer = BuildingAccess {
        gate_code: nonempty_owned(&args.building_access_gate_code)
            .or_else(|| nonempty_owned(&args.gate_code)),
        intercom: nonempty_owned(&args.building_access_intercom),
        note: nonempty_owned(&args.building_access_note),
    };
    if layer.is_empty() {
        None
    } else {
        Some(layer)
    }
}

fn assemble_parking(args: &UpdateConfigArgs) -> Option<ParkingLayer> {
    if args.parking_enabled == Some(false) {
        return None;
    }
    if let Some(structured) = args.parking.clone() {
        if args.parking_enabled != Some(true) && !has_parking_flat(args) {
            return if structured.is_empty() {
                None
            } else {
                Some(structured)
            };
        }
    }
    if args.parking_enabled != Some(true) && !has_parking_flat(args) {
        return None;
    }
    let layer = ParkingLayer {
        info: args.parking_info.trim().to_string(),
        map_url: args.parking_map_url.trim().to_string(),
        code: nonempty_owned(&args.parking_code),
    };
    if layer.is_empty() {
        None
    } else {
        Some(layer)
    }
}

fn has_building_flat(args: &UpdateConfigArgs) -> bool {
    !args.building_access_gate_code.trim().is_empty()
        || !args.building_access_intercom.trim().is_empty()
        || !args.building_access_note.trim().is_empty()
}

fn has_parking_flat(args: &UpdateConfigArgs) -> bool {
    !args.parking_info.trim().is_empty()
        || !args.parking_map_url.trim().is_empty()
        || !args.parking_code.trim().is_empty()
}

fn resolve_smart_lock_provider(args: &UpdateConfigArgs) -> Option<String> {
    nonempty_owned(&args.smart_lock_provider_module_id_custom).or_else(|| {
        args.smart_lock_provider_module_id
            .as_ref()
            .and_then(|s| nonempty_owned(s))
    })
}

fn parse_door_target(raw: &str) -> DoorCodeTarget {
    match raw.trim() {
        "gate" => DoorCodeTarget::Gate,
        "apartment" => DoorCodeTarget::Apartment,
        _ => DoorCodeTarget::Building,
    }
}

fn parse_staff_kind(raw: &str) -> StaffKind {
    match raw.trim() {
        "caretaker" => StaffKind::Caretaker,
        _ => StaffKind::Reception,
    }
}

fn nonempty_owned(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse WGS-84 lat/lng from host form strings. Both required; invalid → none.
fn parse_optional_coord_pair(lat_raw: &str, lng_raw: &str) -> (Option<f64>, Option<f64>) {
    let lat_s = lat_raw.trim();
    let lng_s = lng_raw.trim();
    if lat_s.is_empty() && lng_s.is_empty() {
        return (None, None);
    }
    let Ok(lat) = lat_s.parse::<f64>() else {
        return (None, None);
    };
    let Ok(lng) = lng_s.parse::<f64>() else {
        return (None, None);
    };
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
        return (None, None);
    }
    (Some(lat), Some(lng))
}

fn step_from_input(input: &StepInput, index: usize) -> Option<AccessStep> {
    if input.title_fr.trim().is_empty() && input.title_en.trim().is_empty() {
        return None;
    }
    let kind = input.kind.trim();
    let detail_fr = input.detail_fr.trim();
    let detail_en = input.detail_en.trim();
    Some(AccessStep {
        id: format!("step-{}", index + 1),
        kind: if kind.is_empty() {
            None
        } else {
            Some(kind.to_string())
        },
        title: Localized {
            fr: input.title_fr.trim().to_string(),
            en: input.title_en.trim().to_string(),
        },
        detail: if detail_fr.is_empty() && detail_en.is_empty() {
            None
        } else {
            Some(Localized {
                fr: detail_fr.to_string(),
                en: detail_en.to_string(),
            })
        },
    })
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(_ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    save_config(&args.into_config())
}
