//! Host configuration, held by the platform (`#[portaki_sdk::config]`).
//!
//! [`HostConfig`] is the flat shape the host form sends; the guest surfaces, emails and the
//! readiness check read the nested [`ModuleConfig`] built from it, in one language. Before the
//! platform held it, the module kept a nested blob in KV `config` and its copy per language in
//! `texts/{lang}`: [`legacy`] maps both onto the declared keys.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::hours;
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

    /// Every `value` emitted by the host primary-method ChoiceList, in the spec's order (§2.1).
    /// `host_greets` is no longer offered: a stored one reads as `in_person` (§8).
    pub const CHOICE_LIST_WIRE_VALUES: &[&str] = &[
        "keybox",
        "door_code",
        "smart_lock",
        "in_person",
        "building_staff",
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
    /// « Personnalisé » : `reveal_hours` heures avant l'arrivée (§2.3).
    Custom,
}

impl RevealPolicy {
    /// Wire string for host ChoiceList / `updateConfig` (must match serde rename).
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::HoursBefore24 => "hours_before_24",
            Self::DayBefore16h => "day_before_16h",
            Self::AtCheckin => "at_checkin",
            Self::Custom => "custom",
        }
    }

    /// Every `value` emitted by the host reveal-policy ChoiceList, in the spec's order (§2.3).
    pub const CHOICE_LIST_WIRE_VALUES: &[&str] = &[
        "always",
        "day_before_16h",
        "hours_before_24",
        "at_checkin",
        "custom",
    ];

    pub const ALL: &[RevealPolicy] = &[
        Self::Always,
        Self::DayBefore16h,
        Self::HoursBefore24,
        Self::AtCheckin,
        Self::Custom,
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
    /// « 3e étage, porte de droite » (spec Accès §2.7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// `false` : « Sans ascenseur », utile avec des bagages ; `None` : non précisé.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lift: Option<bool>,
}

impl BuildingAccess {
    pub fn is_empty(&self) -> bool {
        opt_empty(&self.gate_code)
            && opt_empty(&self.intercom)
            && opt_empty(&self.floor)
            && self.lift.is_none()
    }
}

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ParkingLayer {
    #[serde(default)]
    pub map_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// `private`, `street`, `public` ou `garage` (spec Accès §2.8).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Le numéro de place : « 8 ».
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot: Option<String>,
    /// Le tarif, dans la rue ou en parking public : « Gratuit le dimanche, 2 €/h sinon ».
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
}

impl ParkingLayer {
    pub fn is_empty(&self) -> bool {
        self.map_url.trim().is_empty()
            && opt_empty(&self.code)
            && opt_empty(&self.kind)
            && opt_empty(&self.spot)
            && opt_empty(&self.price)
    }
}

/// Le stationnement (§2.8). « Pas de parking » est la case du parking décochée.
pub const PARKING_KINDS: [&str; 4] = ["private", "street", "public", "garage"];
/// L'ascenseur (§2.7), non précisé d'abord : c'est le défaut.
pub const LIFT_CHOICES: [&str; 3] = ["unknown", "yes", "no"];

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
    /// Le bouton « Déverrouiller » (§2.4) : de l'arrivée au départ, ou comme les codes.
    #[serde(default)]
    pub unlock_window: UnlockWindow,
    /// Le créneau de remise des clés (§2.5) : « 16:00 – 19:00 ».
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handover_slot: Option<String>,
    /// Qui remet les clés quand ce n'est pas l'hôte : son nom, et son numéro s'il y en a un.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handover_by: Option<(String, Option<String>)>,
    /// Le téléphone de la réception (§2.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desk_phone: Option<String>,
    /// Que faire en arrivant hors des horaires de la réception (§2.6).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desk_after_hours: Option<String>,
    /// Les horaires de la réception, `mon=07:00-22:00;…` (§2.6) ; vide, l'ancien texte libre.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub desk_hours: String,
    /// « Autre » : comment le voyageur récupère les clés (§2.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_other: Option<String>,
    /// La serrure connectée génère les codes (« Généré par la serrure », §2.2) : le code saisi
    /// n'est alors qu'un code de secours.
    #[serde(default)]
    pub code_by_lock: bool,
    /// « Personnalisé » : combien d'heures avant l'arrivée les codes s'affichent (§2.3).
    #[serde(default = "default_reveal_hours")]
    pub reveal_hours: u32,
    /// Une tâche « Changer le code de la boîte à clés » après chaque départ (§2.2).
    #[serde(default)]
    pub rotate_reminder: bool,
}

/// « Heures avant l'arrivée » : 24 par défaut, de 1 à 168 (§2.3).
pub const DEFAULT_REVEAL_HOURS: u32 = 24;

fn default_reveal_hours() -> u32 {
    DEFAULT_REVEAL_HOURS
}

/// Quand le bouton « Déverrouiller » se montre (§2.4).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnlockWindow {
    /// De l'heure d'arrivée à l'heure de départ.
    #[default]
    Stay,
    /// Comme la révélation des codes.
    Reveal,
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
            unlock_window: UnlockWindow::Stay,
            handover_slot: None,
            handover_by: None,
            desk_phone: None,
            desk_after_hours: None,
            desk_hours: String::new(),
            method_other: None,
            code_by_lock: false,
            reveal_hours: DEFAULT_REVEAL_HOURS,
            rotate_reminder: false,
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
            && opt_empty(&self.method_other)
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

    /// Le code de la barrière du parking, s'il y en a un.
    ///
    /// C'est un secret comme les autres — il suit donc le même calendrier de révélation, et se
    /// montre en tuile avec son bouton Copier plutôt qu'en rangée perdue dans la sous-page.
    pub fn parking_code(&self) -> Option<&str> {
        self.parking
            .as_ref()
            .and_then(|p| p.code.as_deref())
            .map(str::trim)
            .filter(|c| !c.is_empty())
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

    /// Une méthode à code sans son code : le voyageur ne pourra pas entrer. Une serrure liée à
    /// un module émet ses codes elle-même ; le code de secours n'est qu'un repli.
    pub fn entry_code_missing(&self) -> bool {
        match &self.method {
            MethodFields::Keybox { .. } => self.keybox_code().is_none(),
            MethodFields::DoorCode { code, .. } => code.trim().is_empty(),
            // Généré par la serrure : il faut la serrure ; le code saisi n'est qu'un secours.
            MethodFields::SmartLock { .. } if self.code_by_lock => {
                opt_empty(&self.smart_lock_provider_module_id)
            }
            MethodFields::SmartLock { .. } => self.smart_lock_manual_code().is_none(),
            _ => false,
        }
    }

    /// Un code à révéler, quel qu'il soit : celui du moyen d'accès, de l'immeuble, du parking.
    pub fn has_any_code(&self) -> bool {
        self.keybox_code().is_some()
            || self.smart_lock_manual_code().is_some()
            || self.gate_code().is_some()
            || self.parking_code().is_some()
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
    /// Une photo sous l'étape (§2.9) : une URL `https://` ou une référence `portaki-file:`.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub photo: String,
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
            "other"
        ],
        label = "host.method"
    )]
    pub primary_method: String,
    /// « Autre » : comment le voyageur récupère les clés, ≤ 80 (§2.1).
    #[field(label = "host.methodOther")]
    pub method_other: I18nText,
    /// `fixed` : le même code pour tous les séjours ; `lock` : généré par la serrure (§2.2).
    /// Vide : `lock` pour une serrure liée à un module, `fixed` sinon.
    #[field(
        kind = "select",
        options = ["fixed", "lock"],
        label = "host.codeScope"
    )]
    pub code_scope: String,
    /// Une tâche « Changer le code de la boîte à clés » après chaque départ (§2.2).
    #[field(label = "host.rotate")]
    pub rotate_reminder: bool,
    #[field(label = "host.keybox.location")]
    pub keybox_location: I18nText,
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay, arrival_email),
        label = "host.keybox.code"
    )]
    pub keybox_code: String,
    #[field(
        kind = "select",
        options = ["gate", "building", "apartment"],
        label = "host.doorCode.target"
    )]
    pub door_code_target: String,
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay, arrival_email),
        label = "host.doorCode.code"
    )]
    pub door_code: String,
    /// A select whose options are the installed smart-lock modules: free text for the platform.
    #[field(label = "host.smartLock.provider")]
    pub smart_lock_provider_module_id: String,
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay, arrival_email),
        label = "host.smartLock.manualCode"
    )]
    pub smart_lock_manual_code: String,
    #[field(local, label = "host.inPerson.meetingPlace")]
    pub in_person_meeting_place: I18nText,
    #[field(local, label = "host.inPerson.lat")]
    pub in_person_meeting_lat: Option<f64>,
    #[field(local, label = "host.inPerson.lng")]
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
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay, arrival_email),
        label = "host.building.gateCode"
    )]
    pub building_access_gate_code: String,
    #[field(label = "host.building.intercom")]
    pub building_access_intercom: I18nText,
    #[field(label = "host.building.floor")]
    pub building_floor: I18nText,
    #[field(
        kind = "select",
        options = ["unknown", "yes", "no"],
        label = "host.building.lift"
    )]
    pub building_lift: String,
    #[field(label = "host.parking.enabled")]
    pub parking_enabled: bool,
    #[field(kind = "url", label = "host.parking.mapUrl")]
    pub parking_map_url: String,
    #[field(
        kind = "select",
        options = ["private", "street", "public", "garage"],
        label = "host.parking.kind"
    )]
    pub parking_type: String,
    #[field(label = "host.parking.spot")]
    pub parking_spot: String,
    #[field(label = "host.parking.price")]
    pub parking_price: I18nText,
    #[field(
        secret,
        reveal(guest_pre_arrival, guest_stay),
        label = "host.parking.code"
    )]
    pub parking_code: String,
    /// L'entrée du parking (§2.8) : la recherche d'adresse du sélecteur, puis l'épingle.
    #[field(local, label = "host.parking.position")]
    pub parking_address: String,
    #[field(local, label = "host.parking.position")]
    pub parking_lat: Option<f64>,
    #[field(local, label = "host.parking.position")]
    pub parking_lng: Option<f64>,
    #[field(local, label = "host.address.label")]
    pub address: String,
    #[field(local, label = "host.inPerson.lat")]
    pub arrival_lat: Option<f64>,
    #[field(local, label = "host.inPerson.lng")]
    pub arrival_lng: Option<f64>,
    #[field(kind = "url", label = "host.video.label")]
    pub arrival_video_url: String,
    #[field(
        kind = "select",
        options = ["always", "day_before_16h", "hours_before_24", "at_checkin", "custom"],
        label = "config.revealPolicy"
    )]
    pub reveal_policy: String,
    /// « Personnalisé » : 1 à 168 heures avant l'arrivée, 24 par défaut (§2.3).
    #[field(label = "host.reveal.hours")]
    pub reveal_hours: Option<u32>,
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
    /// Ce qu'il faut savoir en arrivant tard : le code du coffre qui change après 22 h, la porte
    /// cochère fermée, le voisin à ne pas réveiller (§2.1).
    ///
    /// Montrée seulement au voyageur qui a **annoncé** une arrivée tardive en pré-arrivée : un
    /// hôte écrit ici une consigne qui ne concerne pas celui qui arrive à 17 h.
    #[field(label = "config.lateArrivalNote")]
    pub late_arrival_note: I18nText,
    /// `stay` (défaut) : de l'arrivée au départ ; `reveal` : comme les codes (§2.4).
    #[field(
        kind = "select",
        options = ["stay", "reveal"],
        label = "host.smartLock.unlockWindow"
    )]
    pub unlock_window: String,
    /// Le créneau de remise des clés, `16:00-19:00` (§2.5).
    #[field(local, label = "host.handover.slot")]
    pub handover_slot: String,
    /// Le créneau d'avant `handover_slot`, en deux heures : lu tant que `handover_slot` est vide.
    #[field(local, label = "host.handover.from")]
    pub handover_slot_from: String,
    #[field(local, label = "host.handover.until")]
    pub handover_slot_until: String,
    #[field(
        kind = "select",
        options = ["me", "other"],
        label = "host.handover.person"
    )]
    pub handover_person: String,
    #[field(label = "host.handover.name")]
    pub handover_name: String,
    #[field(label = "host.handover.phone")]
    pub handover_phone: String,
    #[field(label = "host.desk.phone")]
    pub desk_phone: String,
    #[field(label = "host.desk.afterHours")]
    pub desk_after_hours: I18nText,
    /// Les horaires de la réception, `mon=07:00-22:00;…` (§2.6). Vide : l'ancien texte libre
    /// `building_staff_hours`, montré tel quel.
    #[field(local, label = "host.desk.hours")]
    pub desk_hours: String,
}

/// Combien d'étapes le chemin jusqu'à la porte accepte (spec Accès §2.9).
pub const MAX_STEPS: usize = 8;

impl HostConfig {
    /// Ce qui ne va pas, champ par champ (`keybox_location`, `steps.<i>.title`…) — sous le champ
    /// dans le formulaire, et dans `publishReadiness`. Les codes n'y sont pas : côté hôte, un
    /// secret revient masqué, et le juger sur son masque le dirait faux à tort.
    pub fn problems(&self) -> Vec<(String, I18nText)> {
        use portaki_sdk::config::check;
        let text = crate::i18n::text;
        let too_long = |value: &I18nText, max: usize| {
            value
                .by_language()
                .find_map(|(_, text)| check::max_chars(text, max))
        };
        let mut problems: Vec<(String, I18nText)> = Vec::new();
        let mut push = |field: String, error: Option<I18nText>| {
            if let Some(error) = error {
                problems.push((field, error));
            }
        };
        // « Autre » demande sa précision (§2.1). Avant elle, c'étaient les consignes : une config
        // qui les a remplies n'est pas bloquée.
        let other = self.method() == Some(PrimaryMethod::Other);
        push(
            "method_other".into(),
            if other && self.method_other.is_blank() && self.method_instructions.is_blank() {
                Some(text("host.other.required"))
            } else {
                too_long(&self.method_other, 80)
            },
        );
        push(
            "method_instructions".into(),
            too_long(&self.method_instructions, 1200),
        );
        if self.reveal() == RevealPolicy::Custom {
            push(
                "reveal_hours".into(),
                self.reveal_hours
                    .is_some_and(|hours| !(1..=168).contains(&hours))
                    .then(|| text("host.reveal.hours.invalid")),
            );
        }
        push(
            "building_access_intercom".into(),
            too_long(&self.building_access_intercom, 40),
        );
        push("building_note".into(), too_long(&self.building_note, 280));
        push(
            "arrival_video_url".into(),
            (!is_video_url(self.arrival_video_url.trim())).then(|| text("host.video.invalid")),
        );
        push(
            "keybox_location".into(),
            too_long(&self.keybox_location, 120),
        );
        push("global_note".into(), too_long(&self.global_note, 280));
        push(
            "late_arrival_note".into(),
            too_long(&self.late_arrival_note, 280),
        );
        push("parking_info".into(), too_long(&self.parking_info, 280));
        push("building_floor".into(), too_long(&self.building_floor, 60));
        push(
            "parking_spot".into(),
            check::max_chars(&self.parking_spot, 20),
        );
        push("parking_price".into(), too_long(&self.parking_price, 60));
        // Champs neufs (§2.5, §2.6) : aucune donnée existante à bloquer, et seulement ceux de la
        // méthode choisie — les autres gardent leur dernière valeur sans la montrer.
        if self.method() == Some(PrimaryMethod::InPerson) {
            push(
                "handover_slot".into(),
                (self.handover_slot().is_err()).then(|| text("host.handover.slot.invalid")),
            );
            if self.handover_by_other() {
                push(
                    "handover_name".into(),
                    if self.handover_name.trim().is_empty() {
                        Some(text("host.handover.name.required"))
                    } else {
                        check::max_chars(&self.handover_name, 60)
                    },
                );
                push(
                    "handover_phone".into(),
                    check::phone(&dialable(&self.handover_phone)),
                );
            }
        }
        if self.method() == Some(PrimaryMethod::BuildingStaff) {
            push("desk_hours".into(), desk_hours_overlap(&self.desk_hours));
            push(
                "desk_phone".into(),
                check::phone(&dialable(&self.desk_phone)),
            );
            push(
                "desk_after_hours".into(),
                too_long(&self.desk_after_hours, 280),
            );
        }
        let steps = self.steps.iter().filter(|s| !s.is_blank()).count();
        push(
            "steps".into(),
            (steps > MAX_STEPS).then(|| text("host.steps.tooMany")),
        );
        for (index, step) in self.steps.iter().enumerate() {
            if step.is_blank() {
                continue;
            }
            push(
                format!("steps.{index}.title"),
                if step.title.is_blank() {
                    Some(text("host.step.title.required"))
                } else {
                    too_long(&step.title, 60)
                },
            );
            push(format!("steps.{index}.detail"), too_long(&step.detail, 280));
        }
        problems
    }

    /// Ce qui mérite un coup d'œil sans bloquer la publication : un « Contact » qui ressemble à
    /// un numéro mal saisi. Le champ est libre (un nom, un e-mail y passent sans message).
    pub fn warnings(&self) -> Vec<(String, I18nText)> {
        let mut warnings: Vec<(String, I18nText)> = [
            ("in_person_contact", &self.in_person_contact),
            ("building_staff_contact", &self.building_staff_contact),
        ]
        .into_iter()
        .filter_map(|(field, value)| phone_error(value).map(|error| (field.to_string(), error)))
        .collect();
        // Les anciens horaires en texte libre restent affichés au voyageur ; la réception n'a pas
        // d'« Ouvert maintenant » tant qu'ils ne sont pas ressaisis en plages.
        if self.method() == Some(PrimaryMethod::BuildingStaff)
            && self.desk_hours.trim().is_empty()
            && !self.building_staff_hours.is_blank()
        {
            warnings.push((
                "desk_hours".into(),
                crate::i18n::text("host.desk.hours.legacy"),
            ));
        }
        // Le tarif, dans la rue ou en parking public (§2.8). Un avertissement : les parkings
        // saisis avant ce champ ne bloquent pas la publication.
        if matches!(self.parking_kind(), Some("street" | "public")) && self.parking_price.is_blank()
        {
            warnings.push((
                "parking_price".into(),
                crate::i18n::text("host.parking.price.missing"),
            ));
        }
        warnings
    }

    /// Le message à afficher sous `field`, s'il y en a un.
    pub fn error_of(&self, field: &str) -> Option<I18nText> {
        self.problems()
            .into_iter()
            .find(|(name, _)| name == field)
            .map(|(_, error)| error)
    }

    /// Un parking (§2.8) : « Pas de parking » l'éteint, un type l'allume ; sans type, l'ancien
    /// interrupteur décide.
    pub fn parking_on(&self) -> bool {
        match self.parking_type.trim() {
            "none" => false,
            kind if PARKING_KINDS.contains(&kind) => true,
            _ => self.parking_enabled,
        }
    }

    /// Le type de stationnement, s'il y a un parking et qu'il est connu.
    pub fn parking_kind(&self) -> Option<&str> {
        let kind = self.parking_type.trim();
        (self.parking_on() && PARKING_KINDS.contains(&kind)).then_some(kind)
    }

    /// L'épingle du parking, s'il y a un parking et qu'elle est posée.
    pub fn parking_point(&self) -> Option<(f64, f64)> {
        self.parking_on()
            .then(|| coord_pair(self.parking_lat, self.parking_lng))
            .flatten()
    }

    /// Les codes générés par la serrure (§2.2) — seulement avec une serrure connectée. Sans
    /// choix enregistré : oui si une serrure est liée, comme avant ce réglage.
    pub fn code_by_lock(&self) -> bool {
        self.method() == Some(PrimaryMethod::SmartLock)
            && match self.code_scope.trim() {
                "lock" => true,
                "fixed" => false,
                _ => !self.smart_lock_provider_module_id.trim().is_empty(),
            }
    }

    /// Le créneau tel que le formulaire l'envoie, `16:00-19:00` ; à défaut l'ancienne paire.
    pub fn handover_slot_raw(&self) -> String {
        let slot = self.handover_slot.trim();
        if !slot.is_empty() {
            return slot.to_string();
        }
        let (from, until) = (
            self.handover_slot_from.trim(),
            self.handover_slot_until.trim(),
        );
        if from.is_empty() && until.is_empty() {
            String::new()
        } else {
            format!("{from}-{until}")
        }
    }

    /// L'épingle à plus de 2 km du logement (§2.8) : un avertissement, jamais un blocage. Sans
    /// logement géocodé, rien à comparer.
    pub fn parking_too_far(&self, property: Option<GeoPoint>) -> bool {
        match (self.parking_point(), property) {
            (Some(pin), Some(home)) => distance_km(pin, (home.lat, home.lng)) > 2.0,
            _ => false,
        }
    }

    /// Une autre personne que l'hôte remet les clés (§2.5).
    pub fn handover_by_other(&self) -> bool {
        self.handover_person.trim() == "other"
    }

    /// Le créneau de remise des clés, « 16:00 – 19:00 » ; `Ok(None)` sans créneau, `Err` s'il ne
    /// tient pas dans la journée d'arrivée : il commence après 6:00, finit avant 23:59, et
    /// commence avant de finir.
    pub(crate) fn handover_slot(&self) -> std::result::Result<Option<String>, ()> {
        let raw = self.handover_slot_raw();
        if raw.is_empty() {
            return Ok(None);
        }
        // `HH:MM` se compare comme du texte. Le créneau tient dans la journée : il ne passe pas
        // minuit, contrairement à une plage ordinaire.
        let (start, end) = hours::parse_range(&raw).ok_or(())?;
        if start.as_str() < "06:00" || start >= end {
            return Err(());
        }
        Ok(Some(format!("{start} – {end}")))
    }

    /// The chosen access method, if the host picked one. `host_greets` (« Vous accueillez »)
    /// is now « Remise des clés en main propre » (§8).
    pub fn method(&self) -> Option<PrimaryMethod> {
        match self.primary_method.trim() {
            "host_greets" => Some(PrimaryMethod::InPerson),
            wire => PrimaryMethod::ALL
                .iter()
                .copied()
                .find(|method| method.as_wire() == wire),
        }
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
                    // « Vous accueillez » disait son heure dans `host_greets_eta_hint`.
                    time_hint: text(&self.in_person_time_hint)
                        .or_else(|| text(&self.host_greets_eta_hint)),
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
                floor: text(&self.building_floor),
                lift: match self.building_lift.trim() {
                    "yes" => Some(true),
                    "no" => Some(false),
                    _ => None,
                },
            }),
            parking: self.parking_on().then(|| ParkingLayer {
                map_url: self.parking_map_url.trim().to_string(),
                code: nonempty(&self.parking_code),
                kind: self.parking_kind().map(str::to_string),
                spot: nonempty(self.parking_spot.trim()),
                // Le tarif ne se demande que dans la rue ou en parking public.
                price: matches!(self.parking_kind(), Some("street" | "public"))
                    .then(|| text(&self.parking_price))
                    .flatten(),
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
            unlock_window: if self.unlock_window.trim() == "reveal" {
                UnlockWindow::Reveal
            } else {
                UnlockWindow::Stay
            },
            handover_slot: (primary_method == PrimaryMethod::InPerson)
                .then(|| self.handover_slot().ok().flatten())
                .flatten(),
            handover_by: (primary_method == PrimaryMethod::InPerson && self.handover_by_other())
                .then(|| nonempty(&self.handover_name))
                .flatten()
                .map(|name| (name, nonempty(&dialable(&self.handover_phone)))),
            desk_phone: (primary_method == PrimaryMethod::BuildingStaff)
                .then(|| nonempty(&dialable(&self.desk_phone)))
                .flatten(),
            desk_after_hours: (primary_method == PrimaryMethod::BuildingStaff)
                .then(|| text(&self.desk_after_hours))
                .flatten(),
            desk_hours: if primary_method == PrimaryMethod::BuildingStaff {
                hours::format_week(&hours::parse_week(&self.desk_hours))
            } else {
                String::new()
            },
            method_other: (primary_method == PrimaryMethod::Other)
                .then(|| text(&self.method_other))
                .flatten(),
            code_by_lock: self.code_by_lock(),
            reveal_hours: self
                .reveal_hours
                .filter(|hours| (1..=168).contains(hours))
                .unwrap_or(DEFAULT_REVEAL_HOURS),
            rotate_reminder: self.rotate_reminder && primary_method == PrimaryMethod::Keybox,
        }
    }

    /// The copy in `locale` (else French, English, any), limited to what the method and the
    /// layers show.
    pub fn texts(&self, locale: &str) -> ModuleTexts {
        let text = |text: &I18nText| text.get(locale).trim().to_string();
        // Les instructions détaillées valent pour toutes les méthodes (« À savoir », §2.10). Un
        // ancien « Vous accueillez » avait sa note de contact à la place.
        let greets = self.primary_method.trim() == "host_greets";
        ModuleTexts {
            method_instructions: nonempty(self.method_instructions.get(locale)).or_else(|| {
                greets
                    .then(|| nonempty(self.host_greets_contact_note.get(locale)))
                    .flatten()
            }),
            building_note: self
                .building_access_enabled
                .then(|| nonempty(self.building_note.get(locale)))
                .flatten(),
            parking_info: if self.parking_on() {
                text(&self.parking_info)
            } else {
                String::new()
            },
            global_note: text(&self.global_note),
            late_arrival_note: text(&self.late_arrival_note),
            steps: self
                .live_steps()
                .map(|row| StepText {
                    id: row.id.clone(),
                    kind: row.kind.as_deref().and_then(nonempty),
                    title: text(&row.title),
                    detail: nonempty(row.detail.get(locale)),
                    photo: nonempty(&row.photo),
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

/// La distance à vol d'oiseau (haversine), en kilomètres.
fn distance_km((lat1, lng1): (f64, f64), (lat2, lng2): (f64, f64)) -> f64 {
    let (dlat, dlng) = ((lat2 - lat1).to_radians(), (lng2 - lng1).to_radians());
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    2.0 * 6371.0 * a.sqrt().asin()
}

/// « Deux plages se chevauchent le lundi. » (§2.6), le jour dans la langue du message.
fn desk_hours_overlap(raw: &str) -> Option<I18nText> {
    let week = hours::parse_week(raw);
    let day = hours::DAYS
        .iter()
        .zip(&week)
        .find(|(_, ranges)| hours::overlaps(ranges))?
        .0;
    let name = crate::i18n::text(&format!("day.{day}"));
    // « le mardi » : en français, le jour garde sa minuscule au milieu d'une phrase.
    let message = |lang: &str| {
        let day = match lang {
            "fr" => name.get(lang).to_lowercase(),
            _ => name.get(lang).to_string(),
        };
        crate::i18n::text_vars("host.desk.hours.overlap", &[("day", &day)])
            .get(lang)
            .to_string()
    };
    Some(I18nText::new(message("fr"), message("en")))
}

/// Un lien YouTube, Vimeo, Google Drive ou un fichier `.mp4` (§2.10) ; vide, rien à dire.
fn is_video_url(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return url.is_empty();
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = host.to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    let path = rest.split(['?', '#']).next().unwrap_or_default();
    matches!(
        host,
        "youtube.com"
            | "m.youtube.com"
            | "youtu.be"
            | "vimeo.com"
            | "player.vimeo.com"
            | "drive.google.com"
    ) || path.to_ascii_lowercase().ends_with(".mp4")
}

/// E.164, ou un numéro court (« 3237 ») comme dans `emergency-contacts` : on l'appelle tel quel.
/// Seulement pour une tentative de numéro : des chiffres, ni lettre ni `@`.
fn phone_error(phone: &str) -> Option<I18nText> {
    let attempt = phone.chars().any(|c| c.is_ascii_digit())
        && !phone.chars().any(|c| c.is_alphabetic() || c == '@');
    if !attempt {
        return None;
    }
    let phone = dialable(phone);
    let short = (2..=6).contains(&phone.len()) && phone.bytes().all(|b| b.is_ascii_digit());
    (!short)
        .then(|| portaki_sdk::config::check::phone(&phone))
        .flatten()
}

/// Le numéro tel qu'il se compose : sans espace ni séparateur — « +33 6 12 34 56 78 » tel quel
/// ne passe ni la vérification E.164 ni un lien `tel:`.
pub(crate) fn dialable(phone: &str) -> String {
    phone
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '.' | '-' | '(' | ')'))
        .collect()
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
///
/// An unreadable blob is an error, never a panic: `legacyConfig` fails and the platform retries
/// the import, `load` fails — no empty config is imported over the codes, and nothing traps.
fn legacy(old: Value) -> std::result::Result<Value, String> {
    let (embedded_fr, embedded_en) = extract_embedded_texts(&old);
    let kept = |lang: &str, embedded: ModuleTexts| {
        let kept = load_texts(lang).map_err(|error| error.to_string())?;
        Ok::<_, String>(if kept.is_empty() { embedded } else { kept })
    };
    let (fr, en) = (kept("fr", embedded_fr)?, kept("en", embedded_en)?);
    let raw: RawConfig =
        serde_json::from_value(old).map_err(|error| format!("config_unreadable: {error}"))?;
    Ok(legacy_keys(&migrate_legacy(raw), &fr, &en))
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
        ..ModuleConfig::default()
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
        ..ModuleConfig::default()
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
                ..BuildingAccess::default()
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
        ..ParkingLayer::default()
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
        for wire in PrimaryMethod::CHOICE_LIST_WIRE_VALUES {
            let parsed: PrimaryMethod = serde_json::from_value(json!(wire)).unwrap_or_else(|e| {
                panic!("ChoiceList primary_method value {wire:?} must deserialize: {e}")
            });
            assert_eq!(parsed.as_wire(), *wire);
        }
        // « Vous accueillez » n'est plus offert : il se lit « Remise en main propre » (§8).
        for method in PrimaryMethod::ALL
            .iter()
            .filter(|m| **m != PrimaryMethod::HostGreets)
        {
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
            ..ModuleConfig::default()
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
        mock.run(|_| legacy(old)).expect("a readable legacy blob")
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

    /// Nothing to import rather than an empty config over the codes: the import is retried. An
    /// error, not a panic — a panic traps the module (`legacyConfig`, every render that loads).
    #[test]
    #[serial_test::serial]
    fn an_unreadable_legacy_blob_is_an_error_not_a_trap() {
        let error = portaki_test_utils::MockContext::host()
            .run(|_| legacy(json!({ "keybox_code": "1", "reveal_policy": 3 })))
            .unwrap_err();
        assert!(error.contains("config_unreadable"), "{error}");
        // Already in the declared shape (texts per language): not an old blob either.
        let declared =
            json!({ "primary_method": "keybox", "global_note": { "fr": "A", "en": "B" } });
        portaki_test_utils::MockContext::guest()
            .with_kv("config", serde_json::to_vec(&declared).unwrap())
            .run(|ctx| {
                let error = HostConfig::load(&ctx).unwrap_err().to_string();
                assert!(error.contains("config_unreadable"), "{error}");
            });
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
        // Les instructions détaillées valent pour toutes les méthodes (§2.10).
        assert_eq!(
            in_person.texts("en").method_instructions.as_deref(),
            Some("Turn left")
        );
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

    /// « Autre » demande sa précision ; une étape commencée demande son titre ; les longueurs
    /// de la spec, chacune sur son champ.
    #[test]
    fn host_problems_name_their_field() {
        let config: HostConfig = serde_json::from_value(serde_json::json!({
            "primary_method": "other",
            "keybox_location": "x".repeat(121),
            "steps": [
                { "title": "" , "detail": "" },
                { "title": "", "detail": "Tout droit" }
            ]
        }))
        .unwrap();
        let fields: Vec<String> = config.problems().into_iter().map(|(f, _)| f).collect();
        assert_eq!(fields, ["method_other", "keybox_location", "steps.1.title"]);
    }

    /// L'étage, l'ascenseur, le stationnement : le tarif ne passe qu'en rue ou en parking public.
    #[test]
    fn building_and_parking_reach_the_model() {
        let config: HostConfig = serde_json::from_value(serde_json::json!({
            "building_access_enabled": true,
            "building_floor": "3e étage",
            "building_lift": "no",
            "parking_enabled": true,
            "parking_type": "private",
            "parking_spot": "8",
            "parking_price": "2 €/h"
        }))
        .unwrap();
        let model = config.to_model("fr");
        let building = model.building_access.unwrap();
        assert_eq!(building.floor.as_deref(), Some("3e étage"));
        assert_eq!(building.lift, Some(false));
        let parking = model.parking.unwrap();
        assert_eq!(parking.kind.as_deref(), Some("private"));
        assert_eq!(parking.spot.as_deref(), Some("8"));
        assert_eq!(parking.price, None, "pas de tarif pour une place privée");
    }

    fn problem_of(config: serde_json::Value, field: &str) -> Option<String> {
        let config: HostConfig = serde_json::from_value(config).unwrap();
        config
            .error_of(field)
            .map(|error| error.get("fr").to_string())
    }

    #[test]
    fn other_method_accepts_long_instructions() {
        let json = json!({ "primary_method": "other", "method_instructions": "x".repeat(1200) });
        assert_eq!(problem_of(json, "method_instructions"), None);
    }

    #[test]
    fn other_method_requires_its_precision() {
        assert_eq!(
            problem_of(json!({ "primary_method": "other" }), "method_other").as_deref(),
            Some("Décrivez comment le voyageur récupère les clés.")
        );
        // Avant la précision, les consignes en tenaient lieu : elles ne bloquent pas.
        let legacy = json!({ "primary_method": "other", "method_instructions": "Chez la voisine" });
        assert_eq!(problem_of(legacy, "method_other"), None);
        assert_eq!(
            problem_of(
                json!({ "primary_method": "other", "method_other": "x".repeat(81) }),
                "method_other"
            )
            .as_deref(),
            Some("80 caractères au maximum.")
        );
    }

    #[test]
    fn method_instructions_are_1200_chars_at_most_in_every_language() {
        let json = |n| {
            json!({
                "primary_method": "keybox",
                "method_instructions": { "fr": "court", "en": "x".repeat(n) }
            })
        };
        assert_eq!(problem_of(json(1200), "method_instructions"), None);
        assert_eq!(
            problem_of(json(1201), "method_instructions").as_deref(),
            Some("1200 caractères au maximum.")
        );
    }

    #[test]
    fn intercom_is_40_chars_at_most() {
        let json = |n| json!({ "building_access_intercom": "x".repeat(n) });
        assert_eq!(problem_of(json(40), "building_access_intercom"), None);
        assert_eq!(
            problem_of(json(41), "building_access_intercom").as_deref(),
            Some("40 caractères au maximum.")
        );
    }

    #[test]
    fn building_note_is_280_chars_at_most() {
        let json = |n| json!({ "building_note": { "en": "x".repeat(n) } });
        assert_eq!(problem_of(json(280), "building_note"), None);
        assert_eq!(
            problem_of(json(281), "building_note").as_deref(),
            Some("280 caractères au maximum.")
        );
    }

    #[test]
    fn arrival_video_is_youtube_vimeo_drive_or_mp4() {
        let error =
            |url: &str| problem_of(json!({ "arrival_video_url": url }), "arrival_video_url");
        for ok in [
            "",
            "https://www.youtube.com/watch?v=abc",
            "https://youtu.be/abc",
            "https://vimeo.com/123",
            "https://drive.google.com/file/d/abc/view",
            "https://cdn.example.com/arrivee.MP4?v=2",
        ] {
            assert_eq!(error(ok), None, "{ok}");
        }
        for bad in [
            "https://example.com/video",
            "youtube.com/watch",
            "https://evil.com/youtube.com",
        ] {
            assert_eq!(
                error(bad).as_deref(),
                Some("Utilisez un lien YouTube, Vimeo, Google Drive ou un fichier .mp4."),
                "{bad}"
            );
        }
    }

    fn warning_of(config: serde_json::Value, field: &str) -> Option<String> {
        let config: HostConfig = serde_json::from_value(config).unwrap();
        assert!(
            config.problems().iter().all(|(f, _)| f != field),
            "jamais bloquant"
        );
        config
            .warnings()
            .into_iter()
            .find(|(f, _)| f == field)
            .map(|(_, error)| error.get("fr").to_string())
    }

    #[test]
    fn in_person_contact_warns_on_a_mistyped_phone() {
        let warning = |v: &str| warning_of(json!({ "in_person_contact": v }), "in_person_contact");
        assert_eq!(warning("+33 6 12 34 56 78"), None);
        assert_eq!(
            warning("3237"),
            None,
            "un numéro court passe, comme dans emergency-contacts"
        );
        assert_eq!(warning("Marie, la voisine"), None);
        assert_eq!(
            warning("06 12 34 56 78").as_deref(),
            Some("Ce numéro n'est pas valide. Vérifiez l'indicatif.")
        );
    }

    #[test]
    fn building_staff_contact_warns_on_a_mistyped_phone() {
        let warning = |v: &str| {
            warning_of(
                json!({ "building_staff_contact": v }),
                "building_staff_contact",
            )
        };
        assert_eq!(warning("+41 22 123 45 67"), None);
        assert_eq!(warning("accueil@hotel.fr"), None);
        assert_eq!(
            warning("022 123 45 67").as_deref(),
            Some("Ce numéro n'est pas valide. Vérifiez l'indicatif.")
        );
    }

    #[test]
    fn the_handover_slot_fits_the_arrival_day() {
        const SLOT: &str = "Le créneau doit commencer après 6:00 et finir avant 23:59.";
        let slot = |from: &str, until: &str| {
            problem_of(
                json!({ "primary_method": "in_person", "handover_slot_from": from,
                        "handover_slot_until": until }),
                "handover_slot",
            )
        };
        assert_eq!(slot("", ""), None);
        assert_eq!(slot("16:00", "19:00"), None);
        assert_eq!(slot("6:00", "23:59"), None);
        for (from, until) in [
            ("5:30", "9:00"),
            ("19:00", "16:00"),
            ("16h", "19h"),
            ("16:00", ""),
        ] {
            assert_eq!(slot(from, until).as_deref(), Some(SLOT), "{from}–{until}");
        }
        // Une autre méthode garde sa dernière valeur sans la juger.
        assert_eq!(
            problem_of(
                json!({ "primary_method": "keybox", "handover_slot_from": "5:00" }),
                "handover_slot"
            ),
            None
        );
        let config: HostConfig = serde_json::from_value(json!({
            "primary_method": "in_person", "handover_slot_from": "16:00",
            "handover_slot_until": "19:00"
        }))
        .unwrap();
        assert_eq!(
            config.to_model("fr").handover_slot.as_deref(),
            Some("16:00 – 19:00")
        );
    }

    #[test]
    fn someone_else_handing_over_needs_a_name_and_a_valid_phone() {
        let other = |name: &str, phone: &str, field: &str| {
            problem_of(
                json!({ "primary_method": "in_person", "handover_person": "other",
                        "handover_name": name, "handover_phone": phone }),
                field,
            )
        };
        assert_eq!(
            other("", "", "handover_name").as_deref(),
            Some("Indiquez le nom de la personne.")
        );
        assert_eq!(
            other("Paulette", "06 12", "handover_phone").as_deref(),
            Some("Ce numéro n'est pas valide. Vérifiez l'indicatif.")
        );
        assert_eq!(
            other("Paulette", "+33 6 12 34 56 78", "handover_phone"),
            None
        );
        // Moi : ni nom ni numéro demandés.
        assert_eq!(
            problem_of(
                json!({ "primary_method": "in_person", "handover_person": "me" }),
                "handover_name"
            ),
            None
        );
        let config: HostConfig = serde_json::from_value(json!({
            "primary_method": "in_person", "handover_person": "other",
            "handover_name": "Paulette", "handover_phone": "+33 6 12 34 56 78"
        }))
        .unwrap();
        assert_eq!(
            config.to_model("fr").handover_by,
            Some(("Paulette".into(), Some("+33612345678".into())))
        );
    }

    #[test]
    fn the_desk_phone_is_checked_and_its_after_hours_note_is_280_chars() {
        let desk = |config: serde_json::Value, field: &str| {
            let mut config = config;
            config["primary_method"] = json!("building_staff");
            problem_of(config, field)
        };
        assert_eq!(
            desk(json!({ "desk_phone": "04 93" }), "desk_phone").as_deref(),
            Some("Ce numéro n'est pas valide. Vérifiez l'indicatif.")
        );
        assert_eq!(
            desk(json!({ "desk_phone": "+33493000000" }), "desk_phone"),
            None
        );
        assert!(desk(
            json!({ "desk_after_hours": { "fr": "x".repeat(281) } }),
            "desk_after_hours"
        )
        .is_some());
        assert_eq!(
            desk(
                json!({ "desk_after_hours": { "fr": "x".repeat(280) } }),
                "desk_after_hours"
            ),
            None
        );
    }

    #[test]
    fn the_unlock_window_is_the_stay_unless_the_host_picks_the_reveal() {
        let window = |value: &str| {
            let config: HostConfig =
                serde_json::from_value(json!({ "unlock_window": value })).unwrap();
            config.to_model("fr").unlock_window
        };
        assert_eq!(window(""), UnlockWindow::Stay);
        assert_eq!(window("stay"), UnlockWindow::Stay);
        assert_eq!(window("reveal"), UnlockWindow::Reveal);
    }

    #[test]
    fn custom_reveal_hours_are_1_to_168() {
        let hours = |n: u32| {
            problem_of(
                json!({ "reveal_policy": "custom", "reveal_hours": n }),
                "reveal_hours",
            )
        };
        assert_eq!(hours(1), None);
        assert_eq!(hours(168), None);
        assert_eq!(hours(0).as_deref(), Some("Entre 1 et 168 heures."));
        assert_eq!(hours(169).as_deref(), Some("Entre 1 et 168 heures."));
        let model = |value: Value| {
            serde_json::from_value::<HostConfig>(value)
                .unwrap()
                .to_model("fr")
        };
        let custom = model(json!({ "reveal_policy": "custom", "reveal_hours": 6 }));
        assert_eq!(
            (custom.reveal_policy, custom.reveal_hours),
            (RevealPolicy::Custom, 6)
        );
        assert_eq!(model(json!({ "reveal_policy": "custom" })).reveal_hours, 24);
    }

    #[test]
    fn overlapping_desk_hours_name_the_day() {
        let desk = |week: &str| {
            problem_of(
                json!({ "primary_method": "building_staff", "desk_hours": week }),
                "desk_hours",
            )
        };
        assert_eq!(desk("mon=07:00-12:00,14:00-22:00"), None);
        assert_eq!(
            desk("tue=07:00-12:00,11:00-22:00").as_deref(),
            Some("Deux plages se chevauchent le mardi.")
        );
        let config: HostConfig = serde_json::from_value(
            json!({ "primary_method": "building_staff", "desk_hours": "wed=07:00-12:00,11:00-13:00" }),
        )
        .unwrap();
        assert_eq!(
            config.error_of("desk_hours").unwrap().get("en"),
            "Two time ranges overlap on Wednesday."
        );
    }

    /// Les anciens horaires en texte libre : affichés au voyageur, signalés à l'hôte, jamais
    /// bloquants.
    #[test]
    fn free_text_desk_hours_warn_and_still_show() {
        let config: HostConfig = serde_json::from_value(json!({
            "primary_method": "building_staff",
            "building_staff_hours": "7h – 22h"
        }))
        .unwrap();
        assert!(config.problems().is_empty());
        assert!(config
            .warnings()
            .iter()
            .any(|(field, _)| field == "desk_hours"));
        match config.to_model("fr").method {
            MethodFields::BuildingStaff { hours, .. } => {
                assert_eq!(hours.as_deref(), Some("7h – 22h"))
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_parking_kind_switches_the_parking() {
        let on = |value: Value| {
            serde_json::from_value::<HostConfig>(value)
                .unwrap()
                .parking_on()
        };
        assert!(!on(
            json!({ "parking_enabled": true, "parking_type": "none" })
        ));
        assert!(on(json!({ "parking_type": "street" })));
        // Saisi avant les types : l'interrupteur décide encore.
        assert!(on(json!({ "parking_enabled": true })));
        assert!(!on(json!({})));
        let warnings = |value: Value| {
            serde_json::from_value::<HostConfig>(value)
                .unwrap()
                .warnings()
                .into_iter()
                .map(|(field, _)| field)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            warnings(json!({ "parking_type": "public" })),
            ["parking_price"]
        );
        assert!(warnings(json!({ "parking_type": "public", "parking_price": "2 €/h" })).is_empty());
        assert!(warnings(json!({ "parking_type": "private" })).is_empty());
    }

    #[test]
    fn the_handover_slot_reads_the_range_then_the_old_pair() {
        let slot = |value: Value| {
            serde_json::from_value::<HostConfig>(value)
                .unwrap()
                .to_model("fr")
                .handover_slot
        };
        assert_eq!(
            slot(
                json!({ "primary_method": "in_person", "handover_slot": "16:00-19:00",
                         "handover_slot_from": "10:00", "handover_slot_until": "11:00" })
            )
            .as_deref(),
            Some("16:00 – 19:00")
        );
        assert_eq!(
            problem_of(
                json!({ "primary_method": "in_person", "handover_slot": "22:00-02:00" }),
                "handover_slot"
            )
            .as_deref(),
            Some("Le créneau doit commencer après 6:00 et finir avant 23:59.")
        );
    }

    /// Généré par la serrure : il faut la serrure ; le même code pour tous : il faut le code.
    #[test]
    fn the_code_scope_decides_what_is_missing() {
        let missing = |value: Value| {
            serde_json::from_value::<HostConfig>(value)
                .unwrap()
                .to_model("fr")
                .entry_code_missing()
        };
        let lock = json!({ "primary_method": "smart_lock", "code_scope": "lock" });
        assert!(missing(lock));
        assert!(!missing(
            json!({ "primary_method": "smart_lock", "code_scope": "lock",
                                 "smart_lock_provider_module_id": "nuki" })
        ));
        assert!(missing(
            json!({ "primary_method": "smart_lock", "code_scope": "fixed",
                                "smart_lock_provider_module_id": "nuki" })
        ));
        // Sans choix : une serrure liée génère ses codes, comme avant ce réglage.
        assert!(!missing(json!({ "primary_method": "smart_lock",
                                 "smart_lock_provider_module_id": "nuki" })));
        // « Généré par la serrure » ne vaut qu'avec une serrure.
        let keybox: HostConfig =
            serde_json::from_value(json!({ "primary_method": "keybox", "code_scope": "lock" }))
                .unwrap();
        assert!(!keybox.code_by_lock());
    }

    #[test]
    fn the_rotation_reminder_is_for_a_key_box() {
        let rotate = |method: &str| {
            serde_json::from_value::<HostConfig>(
                json!({ "primary_method": method, "rotate_reminder": true }),
            )
            .unwrap()
            .to_model("fr")
            .rotate_reminder
        };
        assert!(rotate("keybox"));
        assert!(!rotate("door_code"));
    }
}
