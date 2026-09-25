//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! [`HostConfig`] is the flat shape the host form sends; the guest surfaces, emails and the
//! readiness check read the nested [`ModuleConfig`] built from it. Before the platform held it,
//! the module kept a nested blob in KV `config` and its copy per language in `texts/{lang}`:
//! [`HostConfig::read`] still reads them for every key the platform does not hold yet.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::texts::{extract_embedded_texts, lang_code, load_texts, ModuleTexts, StepText};

// ── Public schema ────────────────────────────────────────────────────────────

#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryMethod {
    Keybox,
    DoorCode,
    SmartLock,
    InPerson,
    BuildingStaff,
    HostGreets,
    #[default]
    Other,
}

impl PrimaryMethod {
    /// Wire string for host ChoiceList / `updateConfig` (must match serde `snake_case`).
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Keybox => "keybox",
            Self::DoorCode => "door_code",
            Self::SmartLock => "smart_lock",
            Self::InPerson => "in_person",
            Self::BuildingStaff => "building_staff",
            Self::HostGreets => "host_greets",
            Self::Other => "other",
        }
    }

    /// Every `value` emitted by the host primary-method ChoiceList.
    pub const CHOICE_LIST_WIRE_VALUES: &[&str] = &[
        "keybox",
        "door_code",
        "smart_lock",
        "in_person",
        "building_staff",
        "host_greets",
        "other",
    ];

    pub const ALL: &[PrimaryMethod] = &[
        Self::Keybox,
        Self::DoorCode,
        Self::SmartLock,
        Self::InPerson,
        Self::BuildingStaff,
        Self::HostGreets,
        Self::Other,
    ];

    /// Whether this primary method can carry an access code / credential.
    ///
    /// No-code methods (`in_person`, `building_staff`, `host_greets`, `other`)
    /// do not — host reveal timing UI is hidden unless a code-bearing layer
    /// (building / parking) is also enabled.
    pub const fn involves_access_code(self) -> bool {
        matches!(self, Self::Keybox | Self::DoorCode | Self::SmartLock)
    }
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DoorCodeTarget {
    Gate,
    #[default]
    Building,
    Apartment,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum StaffKind {
    #[default]
    Reception,
    Caretaker,
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RevealPolicy {
    Always,
    /// Wire: `hours_before_24`. Alias keeps legacy `hours_before24` (`rename_all` form).
    #[serde(rename = "hours_before_24", alias = "hours_before24")]
    HoursBefore24,
    /// Wire: `day_before_16h`. Alias keeps legacy `day_before16h` (`rename_all` form).
    #[default]
    #[serde(rename = "day_before_16h", alias = "day_before16h")]
    DayBefore16h,
    AtCheckin,
}

impl RevealPolicy {
    /// Wire string for host ChoiceList / `updateConfig` (must match serde rename).
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::HoursBefore24 => "hours_before_24",
            Self::DayBefore16h => "day_before_16h",
            Self::AtCheckin => "at_checkin",
        }
    }

    /// Every `value` emitted by the host reveal-policy ChoiceList.
    pub const CHOICE_LIST_WIRE_VALUES: &[&str] =
        &["always", "hours_before_24", "day_before_16h", "at_checkin"];

    pub const ALL: &[RevealPolicy] = &[
        Self::Always,
        Self::HoursBefore24,
        Self::DayBefore16h,
        Self::AtCheckin,
    ];
}

/// Fields for the selected primary access method (tagged by `kind`).
/// Text instructions live in [`ModuleTexts::method_instructions`].
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MethodFields {
    Keybox {
        #[serde(default)]
        location: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
    DoorCode {
        #[serde(default)]
        target: DoorCodeTarget,
        #[serde(default)]
        code: String,
    },
    SmartLock {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        manual_code: Option<String>,
    },
    InPerson {
        #[serde(default)]
        meeting_place: String,
        /// WGS-84 meeting point (optional; both lat + lng required when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lat: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lng: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        time_hint: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        contact: Option<String>,
    },
    BuildingStaff {
        #[serde(default)]
        staff_kind: StaffKind,
        #[serde(default)]
        desk_location: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hours: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        contact: Option<String>,
    },
    HostGreets {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        contact_note: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        eta_hint: Option<String>,
    },
    Other {},
}

impl Default for MethodFields {
    fn default() -> Self {
        Self::Other {}
    }
}

impl MethodFields {
    pub fn primary_method(&self) -> PrimaryMethod {
        match self {
            Self::Keybox { .. } => PrimaryMethod::Keybox,
            Self::DoorCode { .. } => PrimaryMethod::DoorCode,
            Self::SmartLock { .. } => PrimaryMethod::SmartLock,
            Self::InPerson { .. } => PrimaryMethod::InPerson,
            Self::BuildingStaff { .. } => PrimaryMethod::BuildingStaff,
            Self::HostGreets { .. } => PrimaryMethod::HostGreets,
            Self::Other {} => PrimaryMethod::Other,
        }
    }
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BuildingAccess {
    /// Digicode for gate / building entrance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intercom: Option<String>,
}

impl BuildingAccess {
    pub fn is_empty(&self) -> bool {
        opt_empty(&self.gate_code) && opt_empty(&self.intercom)
    }
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ParkingLayer {
    #[serde(default)]
    pub map_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ParkingLayer {
    pub fn is_empty(&self) -> bool {
        self.map_url.trim().is_empty() && opt_empty(&self.code)
    }
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ArrivalGuide {
    #[serde(default)]
    pub address: String,
    /// Step skeleton (`id` + `kind`); titles/details live in [`ModuleTexts::steps`].
    #[serde(default)]
    pub steps: Vec<AccessStep>,
    #[serde(default)]
    pub arrival_video_url: String,
}

impl ArrivalGuide {
    pub fn is_empty(&self) -> bool {
        self.address.trim().is_empty()
            && self.parse_steps().is_empty()
            && self.arrival_video_url.trim().is_empty()
    }

    pub fn parse_steps(&self) -> Vec<AccessStep> {
        self.steps
            .iter()
            .filter(|s| !s.id.trim().is_empty())
            .cloned()
            .collect()
    }
}

/// Shared step skeleton (language-invariant).
#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessStep {
    pub id: String,
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleConfig {
    #[serde(default)]
    pub primary_method: PrimaryMethod,
    #[serde(default)]
    pub method: MethodFields,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub building_access: Option<BuildingAccess>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parking: Option<ParkingLayer>,
    #[serde(default)]
    pub arrival: ArrivalGuide,
    #[serde(default)]
    pub reveal_policy: RevealPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smart_lock_provider_module_id: Option<String>,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            primary_method: PrimaryMethod::Other,
            method: MethodFields::default(),
            building_access: None,
            parking: None,
            arrival: ArrivalGuide::default(),
            reveal_policy: RevealPolicy::DayBefore16h,
            smart_lock_provider_module_id: None,
        }
    }
}

impl ModuleConfig {
    /// True when shared structure has no codes/layers (texts may still hold copy).
    pub fn is_empty(&self) -> bool {
        method_is_empty(&self.method)
            && self
                .building_access
                .as_ref()
                .map(BuildingAccess::is_empty)
                .unwrap_or(true)
            && self
                .parking
                .as_ref()
                .map(ParkingLayer::is_empty)
                .unwrap_or(true)
            && self.arrival.is_empty()
            && opt_empty(&self.smart_lock_provider_module_id)
            && self.primary_method == PrimaryMethod::Other
    }

    pub fn parse_steps(&self) -> Vec<AccessStep> {
        self.arrival.parse_steps()
    }

    /// Merge shared step skeletons with per-lang titles/details (by `id`).
    pub fn resolve_steps(&self, texts: &ModuleTexts) -> Vec<ResolvedStep> {
        self.parse_steps()
            .into_iter()
            .map(|step| {
                let text = texts.step_by_id(&step.id);
                ResolvedStep {
                    id: step.id,
                    kind: step.kind,
                    title: text.map(|t| t.title.clone()).unwrap_or_default(),
                    detail: text.and_then(|t| t.detail.clone()),
                }
            })
            .filter(|s| !s.id.trim().is_empty())
            .collect()
    }

    /// Align `primary_method` with the tagged `method` variant.
    pub fn sync_primary_method(&mut self) {
        self.primary_method = self.method.primary_method();
    }

    pub fn address(&self) -> &str {
        self.arrival.address.as_str()
    }

    pub fn gate_code(&self) -> Option<&str> {
        match &self.method {
            MethodFields::DoorCode { code, .. } if !code.trim().is_empty() => Some(code.as_str()),
            _ => self
                .building_access
                .as_ref()
                .and_then(|b| b.gate_code.as_deref())
                .filter(|c| !c.trim().is_empty()),
        }
    }

    pub fn keybox_code(&self) -> Option<&str> {
        match &self.method {
            MethodFields::Keybox { code: Some(c), .. } if !c.trim().is_empty() => Some(c.as_str()),
            _ => None,
        }
    }

    /// Manual / fallback code for smart lock (guest redaction applies at render).
    pub fn smart_lock_manual_code(&self) -> Option<&str> {
        match &self.method {
            MethodFields::SmartLock {
                manual_code: Some(c),
            } if !c.trim().is_empty() => Some(c.as_str()),
            _ => None,
        }
    }

    pub fn parking_map_url(&self) -> Option<&str> {
        self.parking
            .as_ref()
            .map(|p| p.map_url.as_str())
            .filter(|s| !s.trim().is_empty())
    }

    pub fn arrival_video_url(&self) -> &str {
        self.arrival.arrival_video_url.as_str()
    }
}

/// Step with shared skeleton + resolved title/detail for one locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStep {
    pub id: String,
    pub kind: Option<String>,
    pub title: String,
    pub detail: Option<String>,
}

impl ModuleConfig {
    /// The config of this install — see [`HostConfig::read`].
    pub fn read(ctx: &Context) -> Result<Self> {
        Ok(HostConfig::read(ctx)?.to_model())
    }
}

// ── Host settings (declared) ─────────────────────────────────────────────────

/// One arrival step as the host form sends it (`steps.N.kind`). The step list blanks a removed
/// row (`kind: ""`); a step imported from the pre-redesign KV may carry no kind at all.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StepRow {
    pub kind: Option<String>,
}

impl StepRow {
    fn is_removed(&self) -> bool {
        self.kind
            .as_deref()
            .is_some_and(|kind| kind.trim().is_empty())
    }
}

/// The copy of the step at the same index in `steps`, in one language (`steps_fr.N.title`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StepTextRow {
    pub title: String,
    pub detail: String,
}

/// The host form, key for key: the platform takes `updateConfig` itself and refuses any other
/// key. Only the fields of the chosen method are shown, so the others keep their last value.
/// Free text is per language (`_fr` / `_en`, the two languages of the dashboard): a save in
/// English must not overwrite the French copy.
#[portaki_sdk::config]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct HostConfig {
    #[field(
        required,
        kind = "select",
        options = [
            "keybox",
            "door_code",
            "smart_lock",
            "in_person",
            "building_staff",
            "host_greets",
            "other"
        ],
        label = "host.method"
    )]
    pub primary_method: String,
    #[field(label = "host.keybox.location")]
    pub keybox_location: String,
    #[field(secret, label = "host.keybox.code")]
    pub keybox_code: String,
    #[field(
        kind = "select",
        options = ["gate", "building", "apartment"],
        label = "host.doorCode.target"
    )]
    pub door_code_target: String,
    #[field(secret, label = "host.doorCode.code")]
    pub door_code: String,
    /// A select whose options are the installed smart-lock modules: free text for the platform.
    #[field(label = "host.smartLock.provider")]
    pub smart_lock_provider_module_id: String,
    #[field(secret, label = "host.smartLock.manualCode")]
    pub smart_lock_manual_code: String,
    #[field(label = "host.inPerson.meetingPlace")]
    pub in_person_meeting_place: String,
    #[field(label = "host.inPerson.lat")]
    pub in_person_meeting_lat: String,
    #[field(label = "host.inPerson.lng")]
    pub in_person_meeting_lng: String,
    #[field(label = "host.inPerson.timeHint")]
    pub in_person_time_hint: String,
    #[field(label = "host.inPerson.contact")]
    pub in_person_contact: String,
    #[field(
        kind = "select",
        options = ["reception", "caretaker"],
        label = "host.buildingStaff.kind"
    )]
    pub building_staff_kind: String,
    #[field(label = "host.buildingStaff.deskLocation")]
    pub building_staff_desk_location: String,
    #[field(label = "host.buildingStaff.hours")]
    pub building_staff_hours: String,
    #[field(label = "host.buildingStaff.contact")]
    pub building_staff_contact: String,
    #[field(kind = "textarea", label = "host.hostGreets.contactNote")]
    pub host_greets_contact_note: String,
    #[field(label = "host.hostGreets.etaHint")]
    pub host_greets_eta_hint: String,
    #[field(label = "host.building.enabled")]
    pub building_access_enabled: bool,
    #[field(secret, label = "host.building.gateCode")]
    pub building_access_gate_code: String,
    #[field(label = "host.building.intercom")]
    pub building_access_intercom: String,
    #[field(label = "host.parking.enabled")]
    pub parking_enabled: bool,
    #[field(label = "host.parking.mapUrl")]
    pub parking_map_url: String,
    #[field(secret, label = "host.parking.code")]
    pub parking_code: String,
    #[field(label = "host.address.label")]
    pub address: String,
    #[field(label = "host.inPerson.lat")]
    pub arrival_lat: String,
    #[field(label = "host.inPerson.lng")]
    pub arrival_lng: String,
    #[field(label = "host.video.label")]
    pub arrival_video_url: String,
    #[field(
        kind = "select",
        options = ["always", "hours_before_24", "day_before_16h", "at_checkin"],
        label = "config.revealPolicy"
    )]
    pub reveal_policy: String,
    #[field(structured, label = "host.steps.label")]
    pub steps: Vec<StepRow>,
    #[field(kind = "textarea", label = "config.methodInstructionsFr")]
    pub method_instructions_fr: String,
    #[field(kind = "textarea", label = "config.methodInstructionsEn")]
    pub method_instructions_en: String,
    #[field(kind = "textarea", label = "config.buildingNoteFr")]
    pub building_note_fr: String,
    #[field(kind = "textarea", label = "config.buildingNoteEn")]
    pub building_note_en: String,
    #[field(kind = "textarea", label = "config.parkingInfoFr")]
    pub parking_info_fr: String,
    #[field(kind = "textarea", label = "config.parkingInfoEn")]
    pub parking_info_en: String,
    #[field(kind = "textarea", label = "config.globalNoteFr")]
    pub global_note_fr: String,
    #[field(kind = "textarea", label = "config.globalNoteEn")]
    pub global_note_en: String,
    #[field(structured, label = "config.stepsFr")]
    pub steps_fr: Vec<StepTextRow>,
    #[field(structured, label = "config.stepsEn")]
    pub steps_en: Vec<StepTextRow>,
}

/// The language of the host form: the dashboard speaks French or English.
pub fn host_lang(locale: &str) -> &'static str {
    if lang_code(locale) == "en" {
        "en"
    } else {
        "fr"
    }
}

impl HostConfig {
    /// The config of this install. The platform imports only the keys it knows from the old KV
    /// blob, which nested most of them (`method`, `arrival`…) and kept the copy apart, in
    /// `texts/{lang}`. So every key the platform does not hold yet is still read from the KV —
    /// all of them before the platform holds the config. A key it holds, even empty, wins.
    pub fn read(ctx: &Context) -> Result<Self> {
        let held = match &ctx.module_config {
            None => return Self::from_legacy(),
            Some(Value::Object(held)) => held.clone(),
            Some(_) => Map::new(),
        };
        let mut merged = match serde_json::to_value(Self::from_legacy()?) {
            Ok(Value::Object(legacy)) => legacy,
            _ => Map::new(),
        };
        merged.extend(held.into_iter().filter(|(_, value)| !value.is_null()));
        serde_json::from_value(Value::Object(merged)).map_err(|error| unreadable(error.to_string()))
    }

    /// The settings as the KV kept them: the `config` blob in any past shape (through
    /// [`migrate_legacy`]) and `texts/fr` / `texts/en`, or the copy the blob still embeds.
    fn from_legacy() -> Result<Self> {
        let raw = portaki_sdk::config::legacy_config()?;
        let (embedded_fr, embedded_en) = extract_embedded_texts(&raw);
        let texts = |lang: &str, embedded: ModuleTexts| -> Result<ModuleTexts> {
            let kept = load_texts(lang)?;
            Ok(if kept.is_empty() { embedded } else { kept })
        };
        let fr = texts("fr", embedded_fr)?;
        let en = texts("en", embedded_en)?;
        let model = match raw {
            Value::Null => None,
            raw => Some(migrate_legacy(
                serde_json::from_value(raw).map_err(|error| unreadable(error.to_string()))?,
            )),
        };
        Ok(Self::from_model(model.as_ref(), &fr, &en))
    }

    fn from_model(model: Option<&ModuleConfig>, fr: &ModuleTexts, en: &ModuleTexts) -> Self {
        let mut config = Self::default();
        let steps = model.map(ModuleConfig::parse_steps).unwrap_or_default();
        if let Some(model) = model {
            config.primary_method = model.primary_method.as_wire().into();
            match &model.method {
                MethodFields::Keybox { location, code } => {
                    config.keybox_location = location.clone();
                    config.keybox_code = code.clone().unwrap_or_default();
                }
                MethodFields::DoorCode { target, code } => {
                    config.door_code_target = door_target_wire(*target).into();
                    config.door_code = code.clone();
                }
                MethodFields::SmartLock { manual_code } => {
                    config.smart_lock_manual_code = manual_code.clone().unwrap_or_default();
                }
                MethodFields::InPerson {
                    meeting_place,
                    lat,
                    lng,
                    time_hint,
                    contact,
                } => {
                    config.in_person_meeting_place = meeting_place.clone();
                    config.in_person_meeting_lat = lat.map(|v| v.to_string()).unwrap_or_default();
                    config.in_person_meeting_lng = lng.map(|v| v.to_string()).unwrap_or_default();
                    config.in_person_time_hint = time_hint.clone().unwrap_or_default();
                    config.in_person_contact = contact.clone().unwrap_or_default();
                }
                MethodFields::BuildingStaff {
                    staff_kind,
                    desk_location,
                    hours,
                    contact,
                } => {
                    config.building_staff_kind = staff_kind_wire(*staff_kind).into();
                    config.building_staff_desk_location = desk_location.clone();
                    config.building_staff_hours = hours.clone().unwrap_or_default();
                    config.building_staff_contact = contact.clone().unwrap_or_default();
                }
                MethodFields::HostGreets {
                    contact_note,
                    eta_hint,
                } => {
                    config.host_greets_contact_note = contact_note.clone().unwrap_or_default();
                    config.host_greets_eta_hint = eta_hint.clone().unwrap_or_default();
                }
                MethodFields::Other {} => {}
            }
            config.smart_lock_provider_module_id = model
                .smart_lock_provider_module_id
                .clone()
                .unwrap_or_default();
            if let Some(building) = &model.building_access {
                config.building_access_gate_code = building.gate_code.clone().unwrap_or_default();
                config.building_access_intercom = building.intercom.clone().unwrap_or_default();
            }
            if let Some(parking) = &model.parking {
                config.parking_map_url = parking.map_url.clone();
                config.parking_code = parking.code.clone().unwrap_or_default();
            }
            config.address = model.arrival.address.clone();
            config.arrival_video_url = model.arrival.arrival_video_url.clone();
            config.reveal_policy = model.reveal_policy.as_wire().into();
        }
        // A layer was on when it held something, its note included.
        config.building_access_enabled = model.is_some_and(|m| m.building_access.is_some())
            || [fr, en].iter().any(|t| !opt_empty(&t.building_note));
        config.parking_enabled = model.is_some_and(|m| m.parking.is_some())
            || [fr, en].iter().any(|t| !t.parking_info.trim().is_empty());
        config.steps = steps
            .iter()
            .map(|step| StepRow {
                kind: step.kind.clone(),
            })
            .collect();
        config.steps_fr = step_text_rows(&steps, fr);
        config.steps_en = step_text_rows(&steps, en);
        config.method_instructions_fr = fr.method_instructions.clone().unwrap_or_default();
        config.method_instructions_en = en.method_instructions.clone().unwrap_or_default();
        config.building_note_fr = fr.building_note.clone().unwrap_or_default();
        config.building_note_en = en.building_note.clone().unwrap_or_default();
        config.parking_info_fr = fr.parking_info.clone();
        config.parking_info_en = en.parking_info.clone();
        config.global_note_fr = fr.global_note.clone();
        config.global_note_en = en.global_note.clone();
        config
    }

    /// The chosen access method, if the host picked one.
    pub fn method(&self) -> Option<PrimaryMethod> {
        PrimaryMethod::ALL
            .iter()
            .copied()
            .find(|method| method.as_wire() == self.primary_method.trim())
    }

    /// The reveal policy; the pre-rename spellings (`hours_before24`) still parse.
    pub fn reveal(&self) -> RevealPolicy {
        serde_json::from_value(Value::String(self.reveal_policy.trim().into())).unwrap_or_default()
    }

    /// The nested model the guest surfaces, emails and readiness check read.
    pub fn to_model(&self) -> ModuleConfig {
        let primary_method = self.method().unwrap_or_default();
        let method = match primary_method {
            PrimaryMethod::Keybox => MethodFields::Keybox {
                location: self.keybox_location.trim().to_string(),
                code: nonempty(&self.keybox_code),
            },
            PrimaryMethod::DoorCode => MethodFields::DoorCode {
                target: match self.door_code_target.trim() {
                    "gate" => DoorCodeTarget::Gate,
                    "apartment" => DoorCodeTarget::Apartment,
                    _ => DoorCodeTarget::Building,
                },
                code: self.door_code.trim().to_string(),
            },
            PrimaryMethod::SmartLock => MethodFields::SmartLock {
                manual_code: nonempty(&self.smart_lock_manual_code),
            },
            PrimaryMethod::InPerson => {
                let (lat, lng) =
                    coord_pair(&self.in_person_meeting_lat, &self.in_person_meeting_lng);
                MethodFields::InPerson {
                    meeting_place: self.in_person_meeting_place.trim().to_string(),
                    lat,
                    lng,
                    time_hint: nonempty(&self.in_person_time_hint),
                    contact: nonempty(&self.in_person_contact),
                }
            }
            PrimaryMethod::BuildingStaff => MethodFields::BuildingStaff {
                staff_kind: if self.building_staff_kind.trim() == "caretaker" {
                    StaffKind::Caretaker
                } else {
                    StaffKind::Reception
                },
                desk_location: self.building_staff_desk_location.trim().to_string(),
                hours: nonempty(&self.building_staff_hours),
                contact: nonempty(&self.building_staff_contact),
            },
            PrimaryMethod::HostGreets => MethodFields::HostGreets {
                contact_note: nonempty(&self.host_greets_contact_note),
                eta_hint: nonempty(&self.host_greets_eta_hint),
            },
            PrimaryMethod::Other => MethodFields::Other {},
        };
        ModuleConfig {
            primary_method,
            method,
            building_access: self.building_access_enabled.then(|| BuildingAccess {
                gate_code: nonempty(&self.building_access_gate_code),
                intercom: nonempty(&self.building_access_intercom),
            }),
            parking: self.parking_enabled.then(|| ParkingLayer {
                map_url: self.parking_map_url.trim().to_string(),
                code: nonempty(&self.parking_code),
            }),
            arrival: ArrivalGuide {
                address: self.address.trim().to_string(),
                steps: self
                    .live_steps()
                    .map(|(index, row)| AccessStep {
                        id: step_id(index),
                        kind: row.kind.as_deref().and_then(nonempty),
                    })
                    .collect(),
                arrival_video_url: self.arrival_video_url.trim().to_string(),
            },
            reveal_policy: self.reveal(),
            smart_lock_provider_module_id: (primary_method == PrimaryMethod::SmartLock)
                .then(|| nonempty(&self.smart_lock_provider_module_id))
                .flatten(),
        }
    }

    /// The copy in `lang` (`fr` or `en`), limited to what the method and the layers show.
    pub fn texts(&self, lang: &str) -> ModuleTexts {
        let en = lang == "en";
        let pick = |fr: &str, other: &str| if en { other } else { fr }.trim().to_string();
        let rows = if en { &self.steps_en } else { &self.steps_fr };
        let has_instructions = matches!(
            self.method().unwrap_or_default(),
            PrimaryMethod::Keybox
                | PrimaryMethod::DoorCode
                | PrimaryMethod::SmartLock
                | PrimaryMethod::Other
        );
        ModuleTexts {
            method_instructions: has_instructions
                .then(|| {
                    nonempty(&pick(
                        &self.method_instructions_fr,
                        &self.method_instructions_en,
                    ))
                })
                .flatten(),
            building_note: self
                .building_access_enabled
                .then(|| nonempty(&pick(&self.building_note_fr, &self.building_note_en)))
                .flatten(),
            parking_info: if self.parking_enabled {
                pick(&self.parking_info_fr, &self.parking_info_en)
            } else {
                String::new()
            },
            global_note: pick(&self.global_note_fr, &self.global_note_en),
            steps: self
                .live_steps()
                .filter_map(|(index, _)| {
                    let row = rows.get(index)?;
                    Some(StepText {
                        id: step_id(index),
                        title: row.title.trim().to_string(),
                        detail: nonempty(&row.detail),
                    })
                })
                .filter(|text| !text.is_empty())
                .collect(),
        }
    }

    /// Guest copy: guest language → property language → `fr` → `en`, the first one written.
    pub fn guest_texts(&self, guest_locale: &str, property_locale: &str) -> ModuleTexts {
        [lang_code(guest_locale), lang_code(property_locale)]
            .into_iter()
            .chain(["fr".to_string(), "en".to_string()])
            .filter(|lang| lang == "fr" || lang == "en")
            .map(|lang| self.texts(&lang))
            .find(|texts| !texts.is_empty())
            .unwrap_or_default()
    }

    /// The steps the host kept, with their index in `steps` (and in `steps_fr` / `steps_en`).
    pub fn live_steps(&self) -> impl Iterator<Item = (usize, &StepRow)> {
        self.steps
            .iter()
            .enumerate()
            .filter(|(_, row)| !row.is_removed())
    }
}

fn step_id(index: usize) -> String {
    format!("step-{}", index + 1)
}

fn step_text_rows(steps: &[AccessStep], texts: &ModuleTexts) -> Vec<StepTextRow> {
    steps
        .iter()
        .map(|step| {
            texts
                .step_by_id(&step.id)
                .map(|text| StepTextRow {
                    title: text.title.clone(),
                    detail: text.detail.clone().unwrap_or_default(),
                })
                .unwrap_or_default()
        })
        .collect()
}

pub fn door_target_wire(target: DoorCodeTarget) -> &'static str {
    match target {
        DoorCodeTarget::Gate => "gate",
        DoorCodeTarget::Building => "building",
        DoorCodeTarget::Apartment => "apartment",
    }
}

pub fn staff_kind_wire(kind: StaffKind) -> &'static str {
    match kind {
        StaffKind::Reception => "reception",
        StaffKind::Caretaker => "caretaker",
    }
}

/// WGS-84 lat/lng from the form strings: both, in range, or none. `0, 0` is none too: the map
/// picker used to send it for a meeting point nobody placed.
pub(crate) fn coord_pair(lat: &str, lng: &str) -> (Option<f64>, Option<f64>) {
    match (lat.trim().parse::<f64>(), lng.trim().parse::<f64>()) {
        (Ok(lat), Ok(lng))
            if (-90.0..=90.0).contains(&lat)
                && (-180.0..=180.0).contains(&lng)
                && (lat, lng) != (0.0, 0.0) =>
        {
            (Some(lat), Some(lng))
        }
        _ => (None, None),
    }
}

fn nonempty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn unreadable(reason: String) -> PortakiError {
    PortakiError::Storage(format!("config_unreadable: {reason}"))
}

// ── Legacy KV shapes ─────────────────────────────────────────────────────────

/// Wire format that accepts both the new schema and pre-redesign flat fields.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub(crate) struct RawConfig {
    pub(crate) primary_method: Option<PrimaryMethod>,
    pub(crate) method: Option<MethodFields>,
    pub(crate) building_access: Option<BuildingAccess>,
    pub(crate) parking: Option<ParkingLayer>,
    pub(crate) arrival: Option<ArrivalGuide>,
    pub(crate) reveal_policy: Option<RevealPolicy>,
    pub(crate) smart_lock_provider_module_id: Option<String>,

    // Legacy flat fields (pre-redesign)
    pub(crate) steps: Vec<RawStep>,
    pub(crate) steps_json: String,
    pub(crate) parking_map_url: String,
    pub(crate) arrival_video_url: String,
    pub(crate) global_note: String,
    pub(crate) address: String,
    pub(crate) gate_code: String,
    pub(crate) keybox_code: String,
    pub(crate) parking_info: String,
}

/// Step skeleton as stored before / during the texts split.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub(crate) struct RawStep {
    pub(crate) id: String,
    pub(crate) kind: Option<String>,
}

impl RawConfig {
    fn has_new_shape(&self) -> bool {
        self.primary_method.is_some() || self.method.is_some() || self.arrival.is_some()
    }

    fn parse_legacy_step_skeletons(&self) -> Vec<AccessStep> {
        if !self.steps.is_empty() {
            return self
                .steps
                .iter()
                .filter(|s| !s.id.trim().is_empty())
                .map(|s| AccessStep {
                    id: s.id.trim().to_string(),
                    kind: s.kind.clone(),
                })
                .collect();
        }
        let raw = self.steps_json.trim();
        if raw.is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<RawStep>>(raw)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !s.id.trim().is_empty())
            .map(|s| AccessStep {
                id: s.id.trim().to_string(),
                kind: s.kind,
            })
            .collect()
    }
}

/// Migrate a raw (possibly legacy) document into the current shared [`ModuleConfig`].
pub(crate) fn migrate_legacy(raw: RawConfig) -> ModuleConfig {
    if raw.has_new_shape() {
        return migrate_new_shape(raw);
    }
    migrate_from_legacy_fields(raw)
}

fn migrate_new_shape(mut raw: RawConfig) -> ModuleConfig {
    let legacy_steps = raw.parse_legacy_step_skeletons();
    let mut arrival = raw.arrival.take().unwrap_or_default();
    if arrival.address.trim().is_empty() && !raw.address.trim().is_empty() {
        arrival.address = raw.address.trim().to_string();
    }
    if arrival.arrival_video_url.trim().is_empty() && !raw.arrival_video_url.trim().is_empty() {
        arrival.arrival_video_url = raw.arrival_video_url.trim().to_string();
    }
    if arrival.steps.is_empty() {
        arrival.steps = legacy_steps;
    } else {
        // Drop any accidentally embedded title/detail by re-mapping skeletons.
        arrival.steps = arrival
            .steps
            .into_iter()
            .filter(|s| !s.id.trim().is_empty())
            .map(|s| AccessStep {
                id: s.id,
                kind: s.kind,
            })
            .collect();
    }

    let mut parking = raw.parking.take();
    if parking.is_none() {
        parking = parking_from_legacy(
            &raw.parking_map_url,
            None,
            !raw.parking_info.trim().is_empty(),
        );
    }

    let (derived_method, derived_building) =
        method_from_legacy_codes(&raw.keybox_code, &raw.gate_code);

    let mut building_access = raw.building_access.take();
    if building_access.is_none() {
        building_access = derived_building;
    }

    let method = raw.method.take().unwrap_or_else(|| {
        derived_method.unwrap_or_else(|| {
            raw.primary_method
                .map(default_method_for)
                .unwrap_or_default()
        })
    });
    let primary_method = raw
        .primary_method
        .unwrap_or_else(|| method.primary_method());

    let mut config = ModuleConfig {
        primary_method,
        method,
        building_access,
        parking,
        arrival,
        reveal_policy: raw.reveal_policy.unwrap_or_default(),
        smart_lock_provider_module_id: nonempty_opt(raw.smart_lock_provider_module_id),
    };
    config.sync_primary_method();
    config
}

fn migrate_from_legacy_fields(raw: RawConfig) -> ModuleConfig {
    let steps = raw.parse_legacy_step_skeletons();
    let (derived_method, mut building_access) =
        method_from_legacy_codes(&raw.keybox_code, &raw.gate_code);

    let method = derived_method.unwrap_or(MethodFields::Other {});
    let primary_method = method.primary_method();

    let parking = parking_from_legacy(
        &raw.parking_map_url,
        None,
        !raw.parking_info.trim().is_empty(),
    );

    let arrival = ArrivalGuide {
        address: raw.address.trim().to_string(),
        steps,
        arrival_video_url: raw.arrival_video_url.trim().to_string(),
    };

    if building_access
        .as_ref()
        .map(BuildingAccess::is_empty)
        .unwrap_or(false)
    {
        building_access = None;
    }

    ModuleConfig {
        primary_method,
        method,
        building_access,
        parking,
        arrival,
        reveal_policy: raw.reveal_policy.unwrap_or(RevealPolicy::DayBefore16h),
        smart_lock_provider_module_id: nonempty_opt(raw.smart_lock_provider_module_id),
    }
}

/// Map legacy `keybox_code` / `gate_code` into primary method + optional building layer.
fn method_from_legacy_codes(
    keybox_code: &str,
    gate_code: &str,
) -> (Option<MethodFields>, Option<BuildingAccess>) {
    let keybox = keybox_code.trim();
    let gate = gate_code.trim();

    if !keybox.is_empty() {
        let building = if !gate.is_empty() {
            Some(BuildingAccess {
                gate_code: Some(gate.to_string()),
                intercom: None,
            })
        } else {
            None
        };
        return (
            Some(MethodFields::Keybox {
                location: String::new(),
                code: Some(keybox.to_string()),
            }),
            building,
        );
    }

    if !gate.is_empty() {
        return (
            Some(MethodFields::DoorCode {
                target: DoorCodeTarget::Building,
                code: gate.to_string(),
            }),
            None,
        );
    }

    (None, None)
}

fn parking_from_legacy(
    map_url: &str,
    code: Option<String>,
    force_layer: bool,
) -> Option<ParkingLayer> {
    let layer = ParkingLayer {
        map_url: map_url.trim().to_string(),
        code: nonempty_opt(code),
    };
    if layer.is_empty() && !force_layer {
        None
    } else {
        Some(layer)
    }
}

fn default_method_for(primary: PrimaryMethod) -> MethodFields {
    match primary {
        PrimaryMethod::Keybox => MethodFields::Keybox {
            location: String::new(),
            code: None,
        },
        PrimaryMethod::DoorCode => MethodFields::DoorCode {
            target: DoorCodeTarget::Building,
            code: String::new(),
        },
        PrimaryMethod::SmartLock => MethodFields::SmartLock { manual_code: None },
        PrimaryMethod::InPerson => MethodFields::InPerson {
            meeting_place: String::new(),
            lat: None,
            lng: None,
            time_hint: None,
            contact: None,
        },
        PrimaryMethod::BuildingStaff => MethodFields::BuildingStaff {
            staff_kind: StaffKind::Reception,
            desk_location: String::new(),
            hours: None,
            contact: None,
        },
        PrimaryMethod::HostGreets => MethodFields::HostGreets {
            contact_note: None,
            eta_hint: None,
        },
        PrimaryMethod::Other => MethodFields::Other {},
    }
}

fn method_is_empty(method: &MethodFields) -> bool {
    match method {
        MethodFields::Keybox { location, code } => location.trim().is_empty() && opt_empty(code),
        MethodFields::DoorCode { code, .. } => code.trim().is_empty(),
        MethodFields::SmartLock { manual_code } => opt_empty(manual_code),
        MethodFields::InPerson {
            meeting_place,
            lat,
            lng,
            time_hint,
            contact,
        } => {
            meeting_place.trim().is_empty()
                && lat.is_none()
                && lng.is_none()
                && opt_empty(time_hint)
                && opt_empty(contact)
        }
        MethodFields::BuildingStaff {
            desk_location,
            hours,
            contact,
            ..
        } => desk_location.trim().is_empty() && opt_empty(hours) && opt_empty(contact),
        MethodFields::HostGreets {
            contact_note,
            eta_hint,
        } => opt_empty(contact_note) && opt_empty(eta_hint),
        MethodFields::Other {} => true,
    }
}

fn opt_empty(value: &Option<String>) -> bool {
    value.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
}

fn nonempty_opt(value: Option<String>) -> Option<String> {
    value.and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

/// True when shared config or locale texts have guest-visible content.
pub fn has_content(config: &ModuleConfig, texts: &ModuleTexts) -> bool {
    !config.is_empty() || !texts.is_empty() || config.primary_method != PrimaryMethod::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texts::StepText;
    use serde_json::json;

    #[test]
    fn migrate_keybox_wins_and_gate_becomes_building_access() {
        let raw = RawConfig {
            keybox_code: "4821".into(),
            gate_code: "A17B".into(),
            parking_info: "Rue A".into(),
            parking_map_url: "https://maps.example.com".into(),
            address: "Ch. des Douaniers".into(),
            arrival_video_url: "https://video.example.com".into(),
            global_note: "Sonnette".into(),
            steps_json: r#"[{"id":"1","kind":"parking","title":{"fr":"Se garer","en":"Park"}}]"#
                .into(),
            ..RawConfig::default()
        };
        let cfg = migrate_legacy(raw);
        assert_eq!(cfg.primary_method, PrimaryMethod::Keybox);
        assert_eq!(cfg.keybox_code(), Some("4821"));
        assert_eq!(
            cfg.building_access
                .as_ref()
                .and_then(|b| b.gate_code.as_deref()),
            Some("A17B")
        );
        assert!(cfg.parking.is_some());
        assert_eq!(cfg.arrival.address, "Ch. des Douaniers");
        assert_eq!(cfg.parse_steps().len(), 1);
        assert_eq!(cfg.parse_steps()[0].id, "1");
        assert_eq!(cfg.parse_steps()[0].kind.as_deref(), Some("parking"));
        assert_eq!(cfg.reveal_policy, RevealPolicy::DayBefore16h);
    }

    #[test]
    fn migrate_gate_only_to_door_code_building() {
        let raw = RawConfig {
            gate_code: "9999".into(),
            ..RawConfig::default()
        };
        let cfg = migrate_legacy(raw);
        assert_eq!(cfg.primary_method, PrimaryMethod::DoorCode);
        match &cfg.method {
            MethodFields::DoorCode { target, code } => {
                assert_eq!(*target, DoorCodeTarget::Building);
                assert_eq!(code, "9999");
            }
            other => panic!("expected DoorCode, got {other:?}"),
        }
        assert!(cfg.building_access.is_none());
    }

    #[test]
    fn migrate_steps_only_to_other() {
        let raw = RawConfig {
            steps: vec![RawStep {
                id: "1".into(),
                kind: Some("door".into()),
            }],
            parking_info: "Sous-sol".into(),
            ..RawConfig::default()
        };
        let cfg = migrate_legacy(raw);
        assert_eq!(cfg.primary_method, PrimaryMethod::Other);
        assert_eq!(cfg.parse_steps().len(), 1);
        assert!(cfg.parking.is_some());
    }

    #[test]
    fn default_reveal_policy_is_day_before_16h() {
        assert_eq!(
            ModuleConfig::default().reveal_policy,
            RevealPolicy::DayBefore16h
        );
    }

    #[test]
    fn reveal_policy_choice_list_values_deserialize() {
        assert_eq!(
            RevealPolicy::CHOICE_LIST_WIRE_VALUES.len(),
            RevealPolicy::ALL.len()
        );
        for wire in RevealPolicy::CHOICE_LIST_WIRE_VALUES {
            let parsed: RevealPolicy = serde_json::from_value(json!(wire)).unwrap_or_else(|e| {
                panic!("ChoiceList reveal_policy value {wire:?} must deserialize: {e}")
            });
            assert_eq!(parsed.as_wire(), *wire);
        }
        for policy in RevealPolicy::ALL {
            assert!(
                RevealPolicy::CHOICE_LIST_WIRE_VALUES.contains(&policy.as_wire()),
                "variant {policy:?} wire {:?} missing from ChoiceList list",
                policy.as_wire()
            );
        }
    }

    #[test]
    fn reveal_policy_serde_round_trip_all_variants() {
        for &policy in RevealPolicy::ALL {
            let value = serde_json::to_value(policy).expect("serialize");
            assert_eq!(value.as_str(), Some(policy.as_wire()));
            let back: RevealPolicy = serde_json::from_value(value).expect("deserialize");
            assert_eq!(back, policy);
        }
    }

    #[test]
    fn reveal_policy_legacy_aliases_still_parse() {
        assert_eq!(
            serde_json::from_value::<RevealPolicy>(json!("hours_before24")).unwrap(),
            RevealPolicy::HoursBefore24
        );
        assert_eq!(
            serde_json::from_value::<RevealPolicy>(json!("day_before16h")).unwrap(),
            RevealPolicy::DayBefore16h
        );
    }

    #[test]
    fn primary_method_choice_list_values_deserialize() {
        assert_eq!(
            PrimaryMethod::CHOICE_LIST_WIRE_VALUES.len(),
            PrimaryMethod::ALL.len()
        );
        for wire in PrimaryMethod::CHOICE_LIST_WIRE_VALUES {
            let parsed: PrimaryMethod = serde_json::from_value(json!(wire)).unwrap_or_else(|e| {
                panic!("ChoiceList primary_method value {wire:?} must deserialize: {e}")
            });
            assert_eq!(parsed.as_wire(), *wire);
        }
        for method in PrimaryMethod::ALL {
            assert!(
                PrimaryMethod::CHOICE_LIST_WIRE_VALUES.contains(&method.as_wire()),
                "variant {method:?} wire {:?} missing from ChoiceList list",
                method.as_wire()
            );
        }
    }

    #[test]
    fn primary_method_serde_round_trip_all_variants() {
        for &method in PrimaryMethod::ALL {
            let value = serde_json::to_value(method).expect("serialize");
            assert_eq!(value.as_str(), Some(method.as_wire()));
            let back: PrimaryMethod = serde_json::from_value(value).expect("deserialize");
            assert_eq!(back, method);
        }
    }

    #[test]
    fn involves_access_code_for_code_methods_only() {
        assert!(PrimaryMethod::Keybox.involves_access_code());
        assert!(PrimaryMethod::DoorCode.involves_access_code());
        assert!(PrimaryMethod::SmartLock.involves_access_code());
        assert!(!PrimaryMethod::InPerson.involves_access_code());
        assert!(!PrimaryMethod::BuildingStaff.involves_access_code());
        assert!(!PrimaryMethod::HostGreets.involves_access_code());
        assert!(!PrimaryMethod::Other.involves_access_code());
    }

    #[test]
    fn new_shape_roundtrip_json_strips_instructions() {
        let cfg = ModuleConfig {
            primary_method: PrimaryMethod::SmartLock,
            method: MethodFields::SmartLock {
                manual_code: Some("1234".into()),
            },
            building_access: None,
            parking: None,
            arrival: ArrivalGuide::default(),
            reveal_policy: RevealPolicy::HoursBefore24,
            smart_lock_provider_module_id: Some("nuki".into()),
        };
        let bytes = serde_json::to_vec(&cfg).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(value.pointer("/method/instructions").is_none());
        assert!(value.pointer("/arrival/global_note").is_none());
        let raw: RawConfig = serde_json::from_value(value).unwrap();
        let loaded = migrate_legacy(raw);
        assert_eq!(loaded.primary_method, PrimaryMethod::SmartLock);
        assert_eq!(
            loaded.smart_lock_provider_module_id.as_deref(),
            Some("nuki")
        );
        assert_eq!(loaded.reveal_policy, RevealPolicy::HoursBefore24);
    }

    #[test]
    fn resolve_steps_merges_texts_by_id() {
        let cfg = ModuleConfig {
            arrival: ArrivalGuide {
                steps: vec![AccessStep {
                    id: "1".into(),
                    kind: Some("parking".into()),
                }],
                ..ArrivalGuide::default()
            },
            ..ModuleConfig::default()
        };
        let texts = ModuleTexts {
            steps: vec![StepText {
                id: "1".into(),
                title: "Se garer".into(),
                detail: Some("Place résident".into()),
            }],
            ..ModuleTexts::default()
        };
        let resolved = cfg.resolve_steps(&texts);
        assert_eq!(resolved[0].title, "Se garer");
        assert_eq!(resolved[0].detail.as_deref(), Some("Place résident"));
        assert_eq!(resolved[0].kind.as_deref(), Some("parking"));
    }

    #[test]
    fn a_removed_step_drops_with_its_copy_and_the_rest_keeps_theirs() {
        let config = HostConfig {
            steps: vec![
                StepRow {
                    kind: Some("parking".into()),
                },
                StepRow {
                    kind: Some(String::new()),
                },
                StepRow { kind: None },
            ],
            steps_fr: ["Se garer", "Retirée", "Monter"]
                .map(|title| StepTextRow {
                    title: title.into(),
                    detail: String::new(),
                })
                .to_vec(),
            ..HostConfig::default()
        };
        let model = config.to_model();
        let texts = config.texts("fr");
        let resolved = model.resolve_steps(&texts);
        let titles: Vec<&str> = resolved.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, ["Se garer", "Monter"]);
        assert_eq!(resolved[1].kind, None);
    }

    #[test]
    fn copy_follows_the_language_then_falls_back() {
        let config = HostConfig {
            primary_method: "keybox".into(),
            global_note_fr: "Sonnez".into(),
            global_note_en: "Ring".into(),
            method_instructions_en: "Turn left".into(),
            parking_info_fr: "Sous-sol".into(),
            ..HostConfig::default()
        };
        assert_eq!(config.guest_texts("en-GB", "fr-FR").global_note, "Ring");
        assert_eq!(config.guest_texts("de-DE", "fr-FR").global_note, "Sonnez");
        assert_eq!(config.guest_texts("de-DE", "en-US").global_note, "Ring");
        assert_eq!(
            config.texts("en").method_instructions.as_deref(),
            Some("Turn left")
        );
        // A layer switched off hides its copy, as clearing it did before.
        assert_eq!(config.texts("fr").parking_info, "");
        let in_person = HostConfig {
            primary_method: "in_person".into(),
            ..config
        };
        assert_eq!(in_person.texts("en").method_instructions, None);
    }

    #[test]
    fn blank_selects_read_as_their_defaults() {
        let model = HostConfig {
            primary_method: "door_code".into(),
            door_code: " 12 ".into(),
            ..HostConfig::default()
        }
        .to_model();
        assert_eq!(
            model.method,
            MethodFields::DoorCode {
                target: DoorCodeTarget::Building,
                code: "12".into()
            }
        );
        assert_eq!(model.reveal_policy, RevealPolicy::DayBefore16h);
        assert_eq!(HostConfig::default().method(), None);
        assert_eq!(
            HostConfig::default().to_model().primary_method,
            PrimaryMethod::Other
        );
    }

    #[test]
    fn an_unplaced_meeting_point_is_no_point() {
        assert_eq!(coord_pair("0", "0"), (None, None));
        assert_eq!(coord_pair("", ""), (None, None));
        assert_eq!(coord_pair("43.7", "7.26"), (Some(43.7), Some(7.26)));
    }
}
