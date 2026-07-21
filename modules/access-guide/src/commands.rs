//! Module commands — configuration persistence (shared config + locale texts).

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{
    config_from_update_parts, save_config, AccessStep, ArrivalGuide, BuildingAccess,
    DoorCodeTarget, MethodFields, ModuleConfig, ParkingLayer, PrimaryMethod, RawConfig,
    RevealPolicy, StaffKind,
};
use crate::texts::{lang_code, save_texts, ModuleTexts, StepText};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepInput {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub detail: String,
    /// Legacy bilingual host form fields — folded into [`Self::title`] / [`Self::detail`].
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

    // ── Legacy / host-form flat fields ───────────────────────────────────────
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

    /// Optional structured texts (API / tests). Host form flat fields win when empty.
    #[serde(default)]
    pub texts: Option<ModuleTexts>,
}

impl UpdateConfigArgs {
    fn resolve_step_parts(&self) -> (Vec<AccessStep>, Vec<StepText>) {
        if !self.steps.is_empty() {
            let mut skeletons = Vec::new();
            let mut texts = Vec::new();
            for (index, input) in self.steps.iter().enumerate() {
                if let Some((skeleton, text)) = step_from_input(input, index) {
                    skeletons.push(skeleton);
                    if !text.is_empty() {
                        texts.push(text);
                    }
                }
            }
            return (skeletons, texts);
        }
        let raw = self.steps_json.trim();
        if raw.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let parsed: Vec<serde_json::Value> = serde_json::from_str(raw).unwrap_or_default();
        let mut skeletons = Vec::new();
        let mut texts = Vec::new();
        for (index, value) in parsed.into_iter().enumerate() {
            let id = value
                .get("id")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("step-{}", index + 1));
            let kind = value
                .get("kind")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let title = value
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let detail = value
                .get("detail")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            skeletons.push(AccessStep {
                id: id.clone(),
                kind,
            });
            if !title.is_empty() || detail.is_some() {
                texts.push(StepText { id, title, detail });
            }
        }
        (skeletons, texts)
    }

    fn into_config_and_texts(self) -> (ModuleConfig, ModuleTexts) {
        if self.method.is_some() {
            return self.into_structured();
        }
        if let Some(primary) = self.primary_method {
            return self.assemble_host_form(primary);
        }
        self.into_legacy()
    }

    fn into_structured(self) -> (ModuleConfig, ModuleTexts) {
        let (step_skeletons, step_texts) = self.resolve_step_parts();
        let mut texts = self.texts.clone().unwrap_or_default();
        if texts.steps.is_empty() {
            texts.steps = step_texts;
        }
        merge_flat_texts(&mut texts, &self);

        let mut arrival = self.arrival.clone().unwrap_or_default();
        if arrival.steps.is_empty() {
            arrival.steps = step_skeletons;
        }
        if arrival.address.trim().is_empty() && !self.address.trim().is_empty() {
            arrival.address = self.address.trim().to_string();
        }
        if arrival.arrival_video_url.trim().is_empty() && !self.arrival_video_url.trim().is_empty()
        {
            arrival.arrival_video_url = self.arrival_video_url.trim().to_string();
        }

        let config = config_from_update_parts(RawConfig {
            primary_method: self.primary_method,
            method: self.method,
            building_access: self.building_access,
            parking: self.parking,
            arrival: Some(arrival),
            reveal_policy: self.reveal_policy,
            smart_lock_provider_module_id: self.smart_lock_provider_module_id,
            steps: Vec::new(),
            steps_json: String::new(),
            parking_map_url: self.parking_map_url,
            arrival_video_url: self.arrival_video_url,
            global_note: String::new(),
            address: self.address,
            gate_code: self.gate_code,
            keybox_code: self.keybox_code,
            parking_info: String::new(),
        });
        (config, texts)
    }

    fn into_legacy(self) -> (ModuleConfig, ModuleTexts) {
        let (step_skeletons, step_texts) = self.resolve_step_parts();
        let mut texts = ModuleTexts {
            steps: step_texts,
            ..ModuleTexts::default()
        };
        merge_flat_texts(&mut texts, &self);

        let config = config_from_update_parts(RawConfig {
            primary_method: None,
            method: None,
            building_access: None,
            parking: None,
            arrival: None,
            reveal_policy: self.reveal_policy,
            smart_lock_provider_module_id: self.smart_lock_provider_module_id,
            steps: step_skeletons
                .into_iter()
                .map(|s| crate::config::RawStep {
                    id: s.id,
                    kind: s.kind,
                })
                .collect(),
            steps_json: String::new(),
            parking_map_url: self.parking_map_url,
            arrival_video_url: self.arrival_video_url,
            global_note: String::new(),
            address: self.address,
            gate_code: self.gate_code,
            keybox_code: self.keybox_code,
            parking_info: self.parking_info.clone(),
        });
        (config, texts)
    }

    fn assemble_host_form(self, primary: PrimaryMethod) -> (ModuleConfig, ModuleTexts) {
        let method = assemble_method(&self, primary);
        let building_access = assemble_building_access(&self);
        let parking = assemble_parking(&self);
        let (step_skeletons, step_texts) = self.resolve_step_parts();
        let arrival = ArrivalGuide {
            address: self.address.trim().to_string(),
            steps: step_skeletons,
            arrival_video_url: self.arrival_video_url.trim().to_string(),
        };
        let provider = if primary == PrimaryMethod::SmartLock {
            resolve_smart_lock_provider(&self)
        } else {
            None
        };

        let mut texts = self.texts.clone().unwrap_or_default();
        if texts.steps.is_empty() {
            texts.steps = step_texts;
        }
        merge_flat_texts(&mut texts, &self);
        // Method instructions from the active primary form field.
        if texts.method_instructions.is_none() {
            texts.method_instructions = method_instructions_from_form(&self, primary);
        }

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
        (config, texts)
    }
}

fn merge_flat_texts(texts: &mut ModuleTexts, args: &UpdateConfigArgs) {
    if texts.building_note.is_none() {
        texts.building_note = nonempty_owned(&args.building_access_note);
    }
    if texts.parking_info.trim().is_empty() {
        texts.parking_info = args.parking_info.trim().to_string();
    }
    if texts.global_note.trim().is_empty() {
        texts.global_note = args.global_note.trim().to_string();
    }
    if texts.method_instructions.is_none() {
        let from_other = nonempty_owned(&args.other_instructions);
        let from_keybox = nonempty_owned(&args.keybox_instructions);
        let from_door = nonempty_owned(&args.door_code_instructions);
        let from_smart = nonempty_owned(&args.smart_lock_instructions);
        texts.method_instructions = from_other.or(from_keybox).or(from_door).or(from_smart);
    }
}

fn method_instructions_from_form(
    args: &UpdateConfigArgs,
    primary: PrimaryMethod,
) -> Option<String> {
    match primary {
        PrimaryMethod::Keybox => nonempty_owned(&args.keybox_instructions),
        PrimaryMethod::DoorCode => nonempty_owned(&args.door_code_instructions),
        PrimaryMethod::SmartLock => nonempty_owned(&args.smart_lock_instructions),
        PrimaryMethod::Other => nonempty_owned(&args.other_instructions),
        _ => None,
    }
}

fn assemble_method(args: &UpdateConfigArgs, primary: PrimaryMethod) -> MethodFields {
    match primary {
        PrimaryMethod::Keybox => MethodFields::Keybox {
            location: args.keybox_location.trim().to_string(),
            code: nonempty_owned(&args.keybox_code),
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
            }
        }
        PrimaryMethod::SmartLock => MethodFields::SmartLock {
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
        PrimaryMethod::Other => MethodFields::Other {},
    }
}

fn assemble_building_access(args: &UpdateConfigArgs) -> Option<BuildingAccess> {
    if args.building_access_enabled == Some(false) {
        return None;
    }
    if let Some(structured) = args.building_access.clone() {
        if args.building_access_enabled != Some(true) && !has_building_flat(args) {
            return Some(structured);
        }
    }
    if args.building_access_enabled != Some(true) && !has_building_flat(args) {
        return None;
    }
    Some(BuildingAccess {
        gate_code: nonempty_owned(&args.building_access_gate_code)
            .or_else(|| nonempty_owned(&args.gate_code)),
        intercom: nonempty_owned(&args.building_access_intercom),
    })
}

fn assemble_parking(args: &UpdateConfigArgs) -> Option<ParkingLayer> {
    if args.parking_enabled == Some(false) {
        return None;
    }
    if let Some(structured) = args.parking.clone() {
        if args.parking_enabled != Some(true) && !has_parking_flat(args) {
            return Some(structured);
        }
    }
    if args.parking_enabled != Some(true) && !has_parking_flat(args) {
        return None;
    }
    // Keep empty layer when only parking_info text is set (enable marker).
    Some(ParkingLayer {
        map_url: args.parking_map_url.trim().to_string(),
        code: nonempty_owned(&args.parking_code),
    })
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

fn step_from_input(input: &StepInput, index: usize) -> Option<(AccessStep, StepText)> {
    let title = first_nonempty_str(&[
        input.title.as_str(),
        input.title_fr.as_str(),
        input.title_en.as_str(),
    ]);
    let kind = input.kind.trim();
    let detail = first_nonempty_str(&[
        input.detail.as_str(),
        input.detail_fr.as_str(),
        input.detail_en.as_str(),
    ]);
    // Keep skeleton even when title empty if kind is set (host draft slots).
    if title.is_empty() && kind.is_empty() && detail.is_empty() {
        return None;
    }
    if title.is_empty() && kind.is_empty() {
        return None;
    }
    let id = format!("step-{}", index + 1);
    let skeleton = AccessStep {
        id: id.clone(),
        kind: if kind.is_empty() {
            None
        } else {
            Some(kind.to_string())
        },
    };
    let text = StepText {
        id,
        title: title.to_string(),
        detail: if detail.is_empty() {
            None
        } else {
            Some(detail.to_string())
        },
    };
    Some((skeleton, text))
}

fn first_nonempty_str<'a>(candidates: &[&'a str]) -> &'a str {
    for candidate in candidates {
        let trimmed = candidate.trim();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    ""
}

#[portaki_sdk::command(name = "updateConfig")]
pub fn update_config(ctx: Context, args: UpdateConfigArgs) -> Result<()> {
    let lang = lang_code(&ctx.locale);
    let (config, texts) = args.into_config_and_texts();
    save_config(&config)?;
    save_texts(&lang, &texts)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn update_config_args_accepts_choice_list_reveal_policies() {
        for wire in RevealPolicy::CHOICE_LIST_WIRE_VALUES {
            let args: UpdateConfigArgs = serde_json::from_value(json!({
                "primary_method": "keybox",
                "reveal_policy": wire,
                "keybox_location": "À droite",
            }))
            .unwrap_or_else(|e| panic!("updateConfig must accept reveal_policy={wire:?}: {e}"));
            assert_eq!(args.reveal_policy.map(|p| p.as_wire()), Some(*wire));
        }
    }

    #[test]
    fn update_config_args_day_before_16h_and_hours_before_24() {
        let day: UpdateConfigArgs = serde_json::from_value(json!({
            "reveal_policy": "day_before_16h",
            "primary_method": "other",
            "other_instructions": "Sonner"
        }))
        .expect("day_before_16h");
        assert_eq!(day.reveal_policy, Some(RevealPolicy::DayBefore16h));

        let hours: UpdateConfigArgs = serde_json::from_value(json!({
            "reveal_policy": "hours_before_24",
            "primary_method": "keybox",
            "keybox_code": "4821"
        }))
        .expect("hours_before_24");
        assert_eq!(hours.reveal_policy, Some(RevealPolicy::HoursBefore24));
    }

    #[test]
    fn update_config_args_accepts_choice_list_primary_methods() {
        for wire in PrimaryMethod::CHOICE_LIST_WIRE_VALUES {
            let args: UpdateConfigArgs = serde_json::from_value(json!({
                "primary_method": wire,
                "reveal_policy": RevealPolicy::DayBefore16h.as_wire(),
            }))
            .unwrap_or_else(|e| panic!("updateConfig must accept primary_method={wire:?}: {e}"));
            assert_eq!(args.primary_method.map(|m| m.as_wire()), Some(*wire));
        }
    }

    #[test]
    fn method_fields_kind_matches_primary_method_wire() {
        for wire in PrimaryMethod::CHOICE_LIST_WIRE_VALUES {
            let method: MethodFields = serde_json::from_value(json!({ "kind": wire }))
                .unwrap_or_else(|e| panic!("MethodFields kind={wire:?} must deserialize: {e}"));
            assert_eq!(method.primary_method().as_wire(), *wire);
        }
    }

    #[test]
    fn assemble_host_form_splits_texts() {
        let args = UpdateConfigArgs {
            primary_method: Some(PrimaryMethod::Keybox),
            keybox_location: "Porte".into(),
            keybox_code: "4821".into(),
            keybox_instructions: "Tourner".into(),
            building_access_enabled: Some(true),
            building_access_note: "Sonnette".into(),
            parking_enabled: Some(true),
            parking_info: "Rue A".into(),
            global_note: "Note".into(),
            steps: vec![StepInput {
                kind: "parking".into(),
                title: "Se garer".into(),
                detail: "Place".into(),
                ..StepInput::default()
            }],
            ..UpdateConfigArgs::default()
        };
        let (config, texts) = args.into_config_and_texts();
        assert_eq!(config.primary_method, PrimaryMethod::Keybox);
        match &config.method {
            MethodFields::Keybox { location, code } => {
                assert_eq!(location, "Porte");
                assert_eq!(code.as_deref(), Some("4821"));
            }
            other => panic!("expected Keybox, got {other:?}"),
        }
        assert_eq!(texts.method_instructions.as_deref(), Some("Tourner"));
        assert_eq!(texts.building_note.as_deref(), Some("Sonnette"));
        assert_eq!(texts.parking_info, "Rue A");
        assert_eq!(texts.global_note, "Note");
        assert_eq!(texts.steps[0].title, "Se garer");
        assert_eq!(config.parse_steps()[0].kind.as_deref(), Some("parking"));
    }
}
