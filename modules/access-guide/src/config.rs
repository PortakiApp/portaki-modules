//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! [`HostConfig`] is the flat shape the host form sends; the guest surfaces, emails and the
//! readiness check read the nested [`ModuleConfig`] built from it, in one language. Before the
//! platform held it, the module kept a nested blob in KV `config` and its copy per language in
//! `texts/{lang}`: [`legacy`] maps both onto the declared keys.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::texts::{extract_embedded_texts, load_texts, ModuleTexts, StepText};

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
            && self.steps.is_empty()
            && self.arrival_video_url.trim().is_empty()
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

impl ModuleConfig {
    /// The config of this install, its text in the language of `ctx.locale`.
    pub fn read(ctx: &Context) -> Result<Self> {
        Ok(HostConfig::load(ctx)?.to_model(&ctx.locale))
    }
}

// ── Host settings (declared) ─────────────────────────────────────────────────

/// One arrival step. The form sends `kind`, `title` and `detail` (and `id`); the platform keeps
/// the languages the host did not write.
#[portaki_sdk::params]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct StepRow {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub title: I18nText,
    pub detail: I18nText,
}

impl StepRow {
    /// Nothing for the guest to read: a slot the host left, or removed (the step list blanks it).
    pub fn is_blank(&self) -> bool {
        self.title.is_blank() && self.detail.is_blank()
    }
}

/// The host form, key for key: the platform takes `updateConfig` itself and refuses any other
/// key. Only the fields of the chosen method are shown, so the others keep their last value.
/// What the guest reads is an [`I18nText`]: a save writes the host's language only.
#[portaki_sdk::config(legacy = legacy)]
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
    pub keybox_location: I18nText,
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
    pub in_person_meeting_place: I18nText,
    #[field(label = "host.inPerson.lat")]
    pub in_person_meeting_lat: Option<f64>,
    #[field(label = "host.inPerson.lng")]
    pub in_person_meeting_lng: Option<f64>,
    #[field(label = "host.inPerson.timeHint")]
    pub in_person_time_hint: I18nText,
    #[field(label = "host.inPerson.contact")]
    pub in_person_contact: String,
    #[field(
        kind = "select",
        options = ["reception", "caretaker"],
        label = "host.buildingStaff.kind"
    )]
    pub building_staff_kind: String,
    #[field(label = "host.buildingStaff.deskLocation")]
    pub building_staff_desk_location: I18nText,
    #[field(label = "host.buildingStaff.hours")]
    pub building_staff_hours: I18nText,
    #[field(label = "host.buildingStaff.contact")]
    pub building_staff_contact: String,
    #[field(label = "host.hostGreets.contactNote")]
    pub host_greets_contact_note: I18nText,
    #[field(label = "host.hostGreets.etaHint")]
    pub host_greets_eta_hint: I18nText,
    #[field(label = "host.building.enabled")]
    pub building_access_enabled: bool,
    #[field(secret, label = "host.building.gateCode")]
    pub building_access_gate_code: String,
    #[field(label = "host.building.intercom")]
    pub building_access_intercom: I18nText,
    #[field(label = "host.parking.enabled")]
    pub parking_enabled: bool,
    #[field(label = "host.parking.mapUrl")]
    pub parking_map_url: String,
    #[field(secret, label = "host.parking.code")]
    pub parking_code: String,
    #[field(label = "host.address.label")]
    pub address: String,
    #[field(label = "host.inPerson.lat")]
    pub arrival_lat: Option<f64>,
    #[field(label = "host.inPerson.lng")]
    pub arrival_lng: Option<f64>,
    #[field(label = "host.video.label")]
    pub arrival_video_url: String,
    #[field(
        kind = "select",
        options = ["always", "hours_before_24", "day_before_16h", "at_checkin"],
        label = "config.revealPolicy"
    )]
    pub reveal_policy: String,
    #[field(label = "host.steps.label")]
    pub steps: Vec<StepRow>,
    #[field(label = "config.methodInstructions")]
    pub method_instructions: I18nText,
    #[field(label = "config.buildingNote")]
    pub building_note: I18nText,
    #[field(label = "config.parkingInfo")]
    pub parking_info: I18nText,
    #[field(label = "config.globalNote")]
    pub global_note: I18nText,
}

impl HostConfig {
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

    /// The nested model the guest surfaces, emails and readiness check read, its text in
    /// `locale`.
    pub fn to_model(&self, locale: &str) -> ModuleConfig {
        let text = |text: &I18nText| nonempty(text.get(locale));
        let primary_method = self.method().unwrap_or_default();
        let method = match primary_method {
            PrimaryMethod::Keybox => MethodFields::Keybox {
                location: text(&self.keybox_location).unwrap_or_default(),
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
                let point = coord_pair(self.in_person_meeting_lat, self.in_person_meeting_lng);
                MethodFields::InPerson {
                    meeting_place: text(&self.in_person_meeting_place).unwrap_or_default(),
                    lat: point.map(|(lat, _)| lat),
                    lng: point.map(|(_, lng)| lng),
                    time_hint: text(&self.in_person_time_hint),
                    contact: nonempty(&self.in_person_contact),
                }
            }
            PrimaryMethod::BuildingStaff => MethodFields::BuildingStaff {
                staff_kind: if self.building_staff_kind.trim() == "caretaker" {
                    StaffKind::Caretaker
                } else {
                    StaffKind::Reception
                },
                desk_location: text(&self.building_staff_desk_location).unwrap_or_default(),
                hours: text(&self.building_staff_hours),
                contact: nonempty(&self.building_staff_contact),
            },
            PrimaryMethod::HostGreets => MethodFields::HostGreets {
                contact_note: text(&self.host_greets_contact_note),
                eta_hint: text(&self.host_greets_eta_hint),
            },
            PrimaryMethod::Other => MethodFields::Other {},
        };
        ModuleConfig {
            primary_method,
            method,
            building_access: self.building_access_enabled.then(|| BuildingAccess {
                gate_code: nonempty(&self.building_access_gate_code),
                intercom: text(&self.building_access_intercom),
            }),
            parking: self.parking_enabled.then(|| ParkingLayer {
                map_url: self.parking_map_url.trim().to_string(),
                code: nonempty(&self.parking_code),
            }),
            arrival: ArrivalGuide {
                address: self.address.trim().to_string(),
                steps: self
                    .live_steps()
                    .map(|row| AccessStep {
                        id: row.id.clone(),
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

    /// The copy in `locale` (else French, English, any), limited to what the method and the
    /// layers show.
    pub fn texts(&self, locale: &str) -> ModuleTexts {
        let text = |text: &I18nText| text.get(locale).trim().to_string();
        let has_instructions = matches!(
            self.method().unwrap_or_default(),
            PrimaryMethod::Keybox
                | PrimaryMethod::DoorCode
                | PrimaryMethod::SmartLock
                | PrimaryMethod::Other
        );
        ModuleTexts {
            method_instructions: has_instructions
                .then(|| nonempty(self.method_instructions.get(locale)))
                .flatten(),
            building_note: self
                .building_access_enabled
                .then(|| nonempty(self.building_note.get(locale)))
                .flatten(),
            parking_info: if self.parking_enabled {
                text(&self.parking_info)
            } else {
                String::new()
            },
            global_note: text(&self.global_note),
            steps: self
                .live_steps()
                .map(|row| StepText {
                    id: row.id.clone(),
                    kind: row.kind.as_deref().and_then(nonempty),
                    title: text(&row.title),
                    detail: nonempty(row.detail.get(locale)),
                })
                .collect(),
        }
    }

    /// The steps the guest reads, in the host's order.
    pub fn live_steps(&self) -> impl Iterator<Item = &StepRow> {
        self.steps.iter().filter(|row| !row.is_blank())
    }
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

/// A WGS-84 point: both coordinates, in range, or none. `0, 0` is none too: the map picker used
/// to send it for a meeting point nobody placed.
pub(crate) fn coord_pair(lat: Option<f64>, lng: Option<f64>) -> Option<(f64, f64)> {
    let (lat, lng) = (lat?, lng?);
    ((-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng) && (lat, lng) != (0.0, 0.0))
        .then_some((lat, lng))
}

fn nonempty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

// ── Legacy KV (before the platform held the config) ──────────────────────────

/// The old KV blob in any past shape ([`migrate_legacy`]) and its copy — kept in `texts/fr` and
/// `texts/en` (written with the blob, never without it), or still embedded in the blob — mapped
/// onto the declared keys. Runs in the module: for `legacyConfig`, and in `load` before the
/// platform holds the config.
fn legacy(old: Value) -> Value {
    // ponytail: the mapping cannot return an error — an unreadable blob panics, so the platform
    // retries the import and `load` fails, rather than importing an empty config over the codes.
    let (embedded_fr, embedded_en) = extract_embedded_texts(&old);
    let kept = |lang: &str, embedded: ModuleTexts| {
        let kept = load_texts(lang).unwrap_or_else(|error| panic!("{error}"));
        if kept.is_empty() {
            embedded
        } else {
            kept
        }
    };
    let (fr, en) = (kept("fr", embedded_fr), kept("en", embedded_en));
    let raw: RawConfig =
        serde_json::from_value(old).unwrap_or_else(|error| panic!("config_unreadable: {error}"));
    legacy_keys(&migrate_legacy(raw), &fr, &en)
}

/// [`legacy`] once the KV is read. A text of the method (`keybox_location`…) had no language: it
/// stays a plain string, which the platform files under the property's language.
fn legacy_keys(model: &ModuleConfig, fr: &ModuleTexts, en: &ModuleTexts) -> Value {
    let mut out = Map::new();
    put(&mut out, "primary_method", model.primary_method.as_wire());
    match &model.method {
        MethodFields::Keybox { location, code } => {
            put(&mut out, "keybox_location", location.as_str());
            put(&mut out, "keybox_code", code.clone());
        }
        MethodFields::DoorCode { target, code } => {
            put(&mut out, "door_code_target", door_target_wire(*target));
            put(&mut out, "door_code", code.as_str());
        }
        MethodFields::SmartLock { manual_code } => {
            put(&mut out, "smart_lock_manual_code", manual_code.clone());
        }
        MethodFields::InPerson {
            meeting_place,
            lat,
            lng,
            time_hint,
            contact,
        } => {
            put(&mut out, "in_person_meeting_place", meeting_place.as_str());
            put(&mut out, "in_person_meeting_lat", *lat);
            put(&mut out, "in_person_meeting_lng", *lng);
            put(&mut out, "in_person_time_hint", time_hint.clone());
            put(&mut out, "in_person_contact", contact.clone());
        }
        MethodFields::BuildingStaff {
            staff_kind,
            desk_location,
            hours,
            contact,
        } => {
            put(
                &mut out,
                "building_staff_kind",
                staff_kind_wire(*staff_kind),
            );
            put(
                &mut out,
                "building_staff_desk_location",
                desk_location.as_str(),
            );
            put(&mut out, "building_staff_hours", hours.clone());
            put(&mut out, "building_staff_contact", contact.clone());
        }
        MethodFields::HostGreets {
            contact_note,
            eta_hint,
        } => {
            put(&mut out, "host_greets_contact_note", contact_note.clone());
            put(&mut out, "host_greets_eta_hint", eta_hint.clone());
        }
        MethodFields::Other {} => {}
    }
    put(
        &mut out,
        "smart_lock_provider_module_id",
        model.smart_lock_provider_module_id.clone(),
    );
    if let Some(building) = &model.building_access {
        put(
            &mut out,
            "building_access_gate_code",
            building.gate_code.clone(),
        );
        put(
            &mut out,
            "building_access_intercom",
            building.intercom.clone(),
        );
    }
    if let Some(parking) = &model.parking {
        put(&mut out, "parking_map_url", parking.map_url.as_str());
        put(&mut out, "parking_code", parking.code.clone());
    }
    put(&mut out, "address", model.arrival.address.as_str());
    put(
        &mut out,
        "arrival_video_url",
        model.arrival.arrival_video_url.as_str(),
    );
    put(&mut out, "reveal_policy", model.reveal_policy.as_wire());
    // A layer was on when it held something, its note included.
    put(
        &mut out,
        "building_access_enabled",
        model.building_access.is_some() || [fr, en].iter().any(|t| !opt_empty(&t.building_note)),
    );
    put(
        &mut out,
        "parking_enabled",
        model.parking.is_some() || [fr, en].iter().any(|t| !t.parking_info.trim().is_empty()),
    );
    let steps: Vec<Value> = model
        .arrival
        .steps
        .iter()
        .map(|step| {
            let (fr, en) = (fr.step_by_id(&step.id), en.step_by_id(&step.id));
            let detail =
                |text: Option<&StepText>| text.and_then(|t| t.detail.clone()).unwrap_or_default();
            let mut row = Map::new();
            put(&mut row, "id", step.id.as_str());
            put(&mut row, "kind", step.kind.clone());
            put(
                &mut row,
                "title",
                both(
                    fr.map_or("", |t| t.title.as_str()),
                    en.map_or("", |t| t.title.as_str()),
                ),
            );
            put(&mut row, "detail", both(&detail(fr), &detail(en)));
            Value::Object(row)
        })
        .collect();
    if !steps.is_empty() {
        out.insert("steps".into(), Value::Array(steps));
    }
    let optional = |text: &Option<String>| text.clone().unwrap_or_default();
    put(
        &mut out,
        "method_instructions",
        both(
            &optional(&fr.method_instructions),
            &optional(&en.method_instructions),
        ),
    );
    put(
        &mut out,
        "building_note",
        both(&optional(&fr.building_note), &optional(&en.building_note)),
    );
    put(
        &mut out,
        "parking_info",
        both(&fr.parking_info, &en.parking_info),
    );
    put(
        &mut out,
        "global_note",
        both(&fr.global_note, &en.global_note),
    );
    Value::Object(out)
}

/// `{fr, en}` without the blank languages; `null` when both are.
fn both(fr: &str, en: &str) -> Value {
    let texts: Map<String, Value> = [("fr", fr), ("en", en)]
        .into_iter()
        .filter(|(_, text)| !text.trim().is_empty())
        .map(|(lang, text)| (lang.to_string(), Value::from(text.trim())))
        .collect();
    if texts.is_empty() {
        Value::Null
    } else {
        Value::Object(texts)
    }
}

/// Sets `key` unless `value` is `null` or a blank string.
fn put(out: &mut Map<String, Value>, key: &str, value: impl Into<Value>) {
    let value = value.into();
    let blank = value.is_null() || value.as_str().is_some_and(|s| s.trim().is_empty());
    if !blank {
        out.insert(key.to_string(), value);
    }
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
        assert_eq!(cfg.arrival.steps.len(), 1);
        assert_eq!(cfg.arrival.steps[0].id, "1");
        assert_eq!(cfg.arrival.steps[0].kind.as_deref(), Some("parking"));
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
        assert_eq!(cfg.arrival.steps.len(), 1);
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

    /// [`legacy`] as the module runs it: `texts/{lang}` in the KV beside the blob.
    fn mapped(old: Value, texts: &[(&str, Value)]) -> Value {
        let mut mock = portaki_test_utils::MockContext::host();
        for (lang, copy) in texts {
            mock = mock.with_kv(format!("texts/{lang}"), serde_json::to_vec(copy).unwrap());
        }
        mock.run(|_| legacy(old))
    }

    fn flat_blob() -> Value {
        json!({
            "address": "Ch. des Douaniers",
            "gate_code": "A17B",
            "keybox_code": "4821",
            "parking_info": "Résident · rue Aubernon",
            "parking_map_url": "https://maps.example.com",
            "arrival_video_url": "https://video.example.com",
            "global_note": "Sonnette à gauche",
            "steps_json": r#"[{"id":"1","kind":"parking","title":{"fr":"Se garer","en":"Park"},"detail":{"fr":"Place résident","en":"Resident spot"}}]"#
        })
    }

    /// Pre-redesign: flat codes, steps in a JSON string with their copy, plain notes (French).
    #[test]
    #[serial_test::serial]
    fn legacy_flat_blob_with_steps_json() {
        let keys = mapped(flat_blob(), &[]);
        assert_eq!(
            keys,
            json!({
                "primary_method": "keybox",
                "keybox_code": "4821",
                "building_access_enabled": true,
                "building_access_gate_code": "A17B",
                "parking_enabled": true,
                "parking_map_url": "https://maps.example.com",
                "address": "Ch. des Douaniers",
                "arrival_video_url": "https://video.example.com",
                "reveal_policy": "day_before_16h",
                "steps": [{
                    "id": "1",
                    "kind": "parking",
                    "title": { "fr": "Se garer", "en": "Park" },
                    "detail": { "fr": "Place résident", "en": "Resident spot" }
                }],
                "parking_info": { "fr": "Résident · rue Aubernon" },
                "global_note": { "fr": "Sonnette à gauche" }
            })
        );
        let config: HostConfig = serde_json::from_value(keys).unwrap();
        assert_eq!(config.texts("en").steps[0].title, "Park");
        assert_eq!(config.texts("fr").parking_info, "Résident · rue Aubernon");
    }

    /// Pre-redesign too: a `steps` array (a kind, no copy), or a gate code alone.
    #[test]
    #[serial_test::serial]
    fn legacy_flat_steps_array_and_gate_only() {
        assert_eq!(
            mapped(
                json!({ "steps": [{ "id": "1", "kind": "door" }], "parking_info": "Sous-sol" }),
                &[]
            ),
            json!({
                "primary_method": "other",
                "reveal_policy": "day_before_16h",
                "building_access_enabled": false,
                "parking_enabled": true,
                "steps": [{ "id": "1", "kind": "door" }],
                "parking_info": { "fr": "Sous-sol" }
            })
        );
        let gate = mapped(json!({ "gate_code": "9999" }), &[]);
        assert_eq!(gate["primary_method"], "door_code");
        assert_eq!(gate["door_code"], "9999");
        assert_eq!(gate["door_code_target"], "building");
        assert_eq!(gate["building_access_enabled"], false);
    }

    /// The redesigned blob: the method nested, the copy in `texts/{lang}` — matched to the steps
    /// by id — and the pre-rename policy spellings.
    #[test]
    #[serial_test::serial]
    fn legacy_nested_blob_and_its_texts() {
        let blob = |policy: &str| {
            json!({
                "primary_method": "keybox",
                "method": { "kind": "keybox", "location": "Sous le pot", "code": "4821" },
                "arrival": { "address": "Rue X", "steps": [
                    { "id": "a", "kind": "door" }, { "id": "b", "kind": "elevator" }
                ] },
                "reveal_policy": policy
            })
        };
        let texts = |note: &str| {
            json!({
                "global_note": note,
                "method_instructions": format!("{note} ·"),
                "steps": [{ "id": "b", "title": format!("{note} b") }, { "id": "a", "title": note }]
            })
        };
        let keys = mapped(
            blob("hours_before24"),
            &[("fr", texts("Note FR")), ("en", texts("Note EN"))],
        );
        assert_eq!(keys["keybox_location"], "Sous le pot");
        assert_eq!(keys["keybox_code"], "4821");
        assert_eq!(keys["address"], "Rue X");
        assert_eq!(keys["reveal_policy"], "hours_before_24");
        assert_eq!(
            keys["global_note"],
            json!({ "fr": "Note FR", "en": "Note EN" })
        );
        assert_eq!(
            keys["method_instructions"],
            json!({ "fr": "Note FR ·", "en": "Note EN ·" })
        );
        assert_eq!(
            keys["steps"],
            json!([
                { "id": "a", "kind": "door", "title": { "fr": "Note FR", "en": "Note EN" } },
                { "id": "b", "kind": "elevator", "title": { "fr": "Note FR b", "en": "Note EN b" } }
            ])
        );
        assert_eq!(
            mapped(blob("day_before16h"), &[])["reveal_policy"],
            "day_before_16h"
        );
    }

    /// A blob that still embeds its copy (`{fr, en}` or a plain string); `texts/{lang}` wins where
    /// it has something.
    #[test]
    #[serial_test::serial]
    fn legacy_texts_embedded_in_the_blob() {
        let blob = json!({
            "primary_method": "other",
            "method": { "kind": "other", "instructions": { "fr": "Sonner", "en": "Ring" } },
            "building_access": { "gate_code": "1234", "note": { "fr": "Portail", "en": "Gate" } },
            "arrival": {
                "global_note": { "fr": "Note FR", "en": "Note EN" },
                "steps": [{ "id": "1", "kind": "parking",
                            "title": { "fr": "Se garer", "en": "Park" }, "detail": "Place 8" }]
            }
        });
        let keys = mapped(blob.clone(), &[]);
        assert_eq!(
            keys["method_instructions"],
            json!({ "fr": "Sonner", "en": "Ring" })
        );
        assert_eq!(
            keys["building_note"],
            json!({ "fr": "Portail", "en": "Gate" })
        );
        assert_eq!(keys["building_access_gate_code"], "1234");
        assert_eq!(keys["building_access_enabled"], true);
        assert_eq!(
            keys["steps"],
            json!([{ "id": "1", "kind": "parking",
                     "title": { "fr": "Se garer", "en": "Park" }, "detail": { "fr": "Place 8" } }])
        );
        let kept = mapped(blob, &[("fr", json!({ "global_note": "Gardée" }))]);
        assert_eq!(
            kept["global_note"],
            json!({ "fr": "Gardée", "en": "Note EN" })
        );
    }

    /// The methods without a code: their text had no language (a plain string, filed by the
    /// platform under the property's), numbers stay numbers.
    #[test]
    #[serial_test::serial]
    fn legacy_methods_without_codes() {
        let in_person = mapped(
            json!({ "method": { "kind": "in_person", "meeting_place": "Gare", "lat": 43.7,
                                "lng": 7.26, "time_hint": "10 min", "contact": "+33 6" } }),
            &[],
        );
        assert_eq!(in_person["primary_method"], "in_person");
        assert_eq!(in_person["in_person_meeting_place"], "Gare");
        assert_eq!(in_person["in_person_meeting_lat"], 43.7);
        assert_eq!(in_person["in_person_meeting_lng"], 7.26);
        assert_eq!(in_person["in_person_time_hint"], "10 min");
        assert_eq!(in_person["in_person_contact"], "+33 6");
        let staff = mapped(
            json!({ "method": { "kind": "building_staff", "staff_kind": "caretaker",
                                "desk_location": "Loge", "hours": "9-18" } }),
            &[],
        );
        assert_eq!(staff["building_staff_kind"], "caretaker");
        assert_eq!(staff["building_staff_desk_location"], "Loge");
        assert_eq!(staff["building_staff_hours"], "9-18");
        let greets = mapped(
            json!({ "method": { "kind": "host_greets", "contact_note": "Appelez", "eta_hint": "5 min" } }),
            &[],
        );
        assert_eq!(greets["host_greets_contact_note"], "Appelez");
        assert_eq!(greets["host_greets_eta_hint"], "5 min");
        let lock = mapped(
            json!({ "primary_method": "smart_lock", "method": { "kind": "smart_lock", "manual_code": "77" },
                    "smart_lock_provider_module_id": "nuki", "parking": { "map_url": "", "code": "P1" },
                    "building_access": { "intercom": "Apt 3" } }),
            &[],
        );
        assert_eq!(lock["smart_lock_manual_code"], "77");
        assert_eq!(lock["smart_lock_provider_module_id"], "nuki");
        assert_eq!(lock["parking_code"], "P1");
        assert_eq!(lock["building_access_intercom"], "Apt 3");
        let config: HostConfig = serde_json::from_value(in_person).unwrap();
        assert_eq!(config.in_person_meeting_place.get("en"), "Gare");
        assert_eq!(config.in_person_meeting_lat, Some(43.7));
    }

    /// Nothing to import rather than an empty config over the codes: the import is retried.
    #[test]
    #[serial_test::serial]
    #[should_panic(expected = "config_unreadable")]
    fn an_unreadable_legacy_blob_is_not_imported_empty() {
        mapped(json!({ "keybox_code": "1", "reveal_policy": 3 }), &[]);
    }

    #[test]
    #[serial_test::serial]
    fn the_kv_is_read_through_legacy_until_the_platform_holds_the_config() {
        portaki_test_utils::MockContext::guest()
            .with_kv("config", serde_json::to_vec(&flat_blob()).unwrap())
            .run(|ctx| {
                let config = HostConfig::load(&ctx).unwrap();
                assert_eq!(config.keybox_code, "4821");
                assert_eq!(config.texts("en-US").steps[0].title, "Park");
            });
        portaki_test_utils::MockContext::guest()
            .with_kv("config", serde_json::to_vec(&flat_blob()).unwrap())
            .with_config(&json!({}))
            .run(|ctx| assert_eq!(HostConfig::load(&ctx).unwrap(), HostConfig::default()));
    }

    #[test]
    fn a_blank_step_drops_and_the_rest_keep_their_place_and_id() {
        let step = |id: &str, title: &str| StepRow {
            id: id.into(),
            kind: Some("parking".into()),
            title: I18nText::new(title, ""),
            ..StepRow::default()
        };
        let config = HostConfig {
            steps: vec![
                step("a", "Se garer"),
                StepRow {
                    kind: Some(String::new()),
                    ..StepRow::default()
                },
                step("", "Monter"),
            ],
            ..HostConfig::default()
        };
        let steps = config.texts("fr").steps;
        let titles: Vec<&str> = steps.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, ["Se garer", "Monter"]);
        assert_eq!(steps[0].id, "a");
        assert_eq!(steps[0].kind.as_deref(), Some("parking"));
        assert_eq!(config.to_model("fr").arrival.steps.len(), 2);
    }

    #[test]
    fn copy_follows_the_guest_language_then_falls_back() {
        let config = HostConfig {
            primary_method: "keybox".into(),
            global_note: I18nText::new("Sonnez", "Ring"),
            method_instructions: I18nText::new("", "Turn left"),
            parking_info: I18nText::new("Sous-sol", ""),
            ..HostConfig::default()
        };
        assert_eq!(config.texts("en-GB").global_note, "Ring");
        assert_eq!(config.texts("de-DE").global_note, "Sonnez");
        assert_eq!(
            config.texts("fr").method_instructions.as_deref(),
            Some("Turn left")
        );
        // A layer switched off hides its copy, as clearing it did before.
        assert_eq!(config.texts("fr").parking_info, "");
        let in_person = HostConfig {
            primary_method: "in_person".into(),
            in_person_meeting_place: I18nText::new("Gare", "Station"),
            in_person_meeting_lat: Some(43.7),
            in_person_meeting_lng: Some(7.26),
            ..config
        };
        assert_eq!(in_person.texts("en").method_instructions, None);
        assert_eq!(
            in_person.to_model("en-US").method,
            MethodFields::InPerson {
                meeting_place: "Station".into(),
                lat: Some(43.7),
                lng: Some(7.26),
                time_hint: None,
                contact: None,
            }
        );
    }

    #[test]
    fn blank_selects_read_as_their_defaults() {
        let model = HostConfig {
            primary_method: "door_code".into(),
            door_code: " 12 ".into(),
            ..HostConfig::default()
        }
        .to_model("fr");
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
            HostConfig::default().to_model("fr").primary_method,
            PrimaryMethod::Other
        );
    }

    #[test]
    fn an_unplaced_meeting_point_is_no_point() {
        assert_eq!(coord_pair(Some(0.0), Some(0.0)), None);
        assert_eq!(coord_pair(None, Some(7.26)), None);
        assert_eq!(coord_pair(Some(43.7), Some(7.26)), Some((43.7, 7.26)));
    }
}
