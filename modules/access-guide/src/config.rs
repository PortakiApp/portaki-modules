//! Shared structural configuration stored in KV (`config` key).
//!
//! Language-specific copy lives in [`crate::texts`] under `texts/{lang}`.

use portaki_sdk::host;
use portaki_sdk::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::texts::{extract_embedded_texts, seed_texts_if_absent, ModuleTexts};

const CONFIG_KEY: &str = "config";

// ── Public schema ────────────────────────────────────────────────────────────

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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DoorCodeTarget {
    Gate,
    #[default]
    Building,
    Apartment,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum StaffKind {
    #[default]
    Reception,
    Caretaker,
}

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

// ── Load / save / migrate ────────────────────────────────────────────────────

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

fn document_has_embedded_texts(root: &Value) -> bool {
    let (fr, en) = extract_embedded_texts(root);
    !fr.is_empty() || !en.is_empty()
}

/// Load shared config; migrate legacy embeds into `texts/{lang}` once, then rewrite `config`.
pub fn load_config() -> Result<ModuleConfig> {
    let Some(bytes) = host::kv::get(CONFIG_KEY)? else {
        return Ok(ModuleConfig::default());
    };
    let root: Value = serde_json::from_slice(&bytes).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid config JSON: {error}"))
    })?;
    let raw: RawConfig = serde_json::from_value(root.clone()).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("invalid config JSON: {error}"))
    })?;
    let config = migrate_legacy(raw);

    if document_has_embedded_texts(&root) {
        let (fr, en) = extract_embedded_texts(&root);
        seed_texts_if_absent("fr", &fr)?;
        seed_texts_if_absent("en", &en)?;
        // Strip embeds so future loads do not re-seed / overwrite texts.
        save_config(&config)?;
    }

    Ok(config)
}

pub fn save_config(config: &ModuleConfig) -> Result<()> {
    let mut config = config.clone();
    config.sync_primary_method();
    // Keep empty Some(parking/building) as enable markers when only texts fill the layer.
    let bytes = serde_json::to_vec(&config).map_err(|error| {
        portaki_sdk::PortakiError::Storage(format!("config serialize: {error}"))
    })?;
    host::kv::set(CONFIG_KEY, &bytes, None)
}

/// Build [`ModuleConfig`] from host `updateConfig` args (new + legacy fields).
pub fn config_from_update_parts(raw: RawConfig) -> ModuleConfig {
    migrate_legacy(raw)
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
}
