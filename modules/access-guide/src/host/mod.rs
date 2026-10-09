//! Host dashboard surfaces — the access settings, one section per block of the spec (§2.1–2.10).
//!
//! A block that depends on a choice (the method, the reveal, the parking) is left out of the tree
//! until the draft makes the choice: the choice lists re-render the surface (`emitOnChange`), and
//! a `SelectableCard` does not seed its value, so a `visibleWhen` keyed on it would open by
//! default. `visibleWhen` is set as well where it can say it (one value), for the shells that
//! honour it between two renders.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, ChoiceList, Field, FieldHint, Form, Grid, ImageUpload, InlineNotice,
    KeyValue, Link, NumberInput, Page, PhoneInput, RadioGroup, RichTextEditor, SecretInput,
    Section, Select, SelectableCard, Stack, StepList, TextInput, TimeRange, ToggleRow, WeeklyHours,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{
    coord_pair, HostConfig, PrimaryMethod, RevealPolicy, StepRow, DEFAULT_REVEAL_HOURS,
    LIFT_CHOICES, PARKING_KINDS,
};

mod stay;

pub use stay::render_host_stay;

const STEP_SLOTS: usize = crate::config::MAX_STEPS;

/// « 7:00 – 22:00 tous les jours » : les horaires proposés à une réception sans horaires (§2.6).
const DEFAULT_DESK_HOURS: &str = "mon=07:00-22:00;tue=07:00-22:00;wed=07:00-22:00;\
thu=07:00-22:00;fri=07:00-22:00;sat=07:00-22:00;sun=07:00-22:00";

// « Code manquant · séjour de Marie · arrivée dans 2 j » dans À venir : `crate::tasks`.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::DangerTriangle
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::AccessEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Key
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = HostConfig::load(&ctx)?;
    // Les messages sous les champs, lus par les aides de champ plus bas.
    ERRORS.with(|errors| {
        *errors.borrow_mut() = config
            .problems()
            .into_iter()
            .chain(config.warnings())
            .map(|(field, error)| (field, error.get(&ctx.locale).to_string()))
            .collect();
    });
    // Le brouillon d'abord : un clic sur une carte re-rend la surface avec son choix.
    let method = ctx
        .input_str("primary_method")
        .and_then(parse_primary_method)
        .or(config.method())
        .unwrap_or(PrimaryMethod::Keybox);
    let reveal = ctx
        .input_str("reveal_policy")
        .and_then(parse_reveal)
        .unwrap_or_else(|| config.reveal());
    let parking = ctx
        .input_str("parking_type")
        .map(str::to_string)
        .unwrap_or_else(|| parking_choice(&config));
    let parking_on = parking != "none";
    let building_enabled =
        ctx.input_bool("building_access_enabled", config.building_access_enabled);
    let steps_count = draft_steps_count(&ctx, &config);

    let mut sections = vec![method_section(method, &config, &ctx)];
    if method.involves_access_code() {
        sections.push(codes_section(method, &config, &ctx));
    }
    // Les codes de l'immeuble et du parking suivent la même révélation (§2.3).
    if method.involves_access_code() || building_enabled || parking_on {
        sections.push(reveal_section(reveal, &config));
    }
    match method {
        PrimaryMethod::SmartLock => sections.push(lock_section(&config)),
        PrimaryMethod::InPerson => sections.push(handover_section(&config, &ctx)),
        PrimaryMethod::BuildingStaff => sections.push(desk_section(&config, &ctx)),
        _ => {}
    }
    sections.push(building_section(building_enabled, &config, &ctx));
    sections.push(parking_section(&parking, &config, &ctx));
    sections.push(path_section(&config, &ctx, steps_count));
    sections.push(know_section(&config, &ctx));

    // No Save button — the modules drawer saves the form (`updateConfig`, taken by the platform).
    Ok(Surface::new(Page::new().child(Form::new().children(sections))).with_id(MAIN))
}

/// One card of the settings page (§2.2 of the common rules): a `Section`, never a `Card`.
fn section(key: &str, visible_when: Option<&str>, children: Vec<Component>) -> Component {
    let section = Section::new()
        .title(format!("i18n:host.section.{key}"))
        .children(children);
    match visible_when {
        Some(condition) => section.visibleWhen(condition),
        None => section,
    }
    .into()
}

// ── 2.1 Méthode d'arrivée ────────────────────────────────────────────────────

fn method_section(method: PrimaryMethod, config: &HostConfig, ctx: &HostContext) -> Component {
    let choices = [
        (PrimaryMethod::Keybox, "keybox", IconName::Key),
        (PrimaryMethod::DoorCode, "door_code", IconName::Grid),
        (PrimaryMethod::SmartLock, "smart_lock", IconName::Lock),
        (PrimaryMethod::InPerson, "in_person", IconName::Users),
        (
            PrimaryMethod::BuildingStaff,
            "building_staff",
            IconName::Building,
        ),
        (PrimaryMethod::Other, "other", IconName::MoreHorizontal),
    ];
    // A `ChoiceList` in cards, not loose `SelectableCard`s: it seeds its value, so the default
    // « Boîte à clés » is saved without a click.
    let list = ChoiceList::new()
        .name("primary_method")
        .value(method.as_wire())
        .emitOnChange(true)
        .layout(ChoiceListLayout::Cards)
        .choices(
            choices
                .iter()
                .map(|(method, key, icon)| {
                    ChoiceOption::new(method.as_wire(), format!("i18n:host.method.{key}"))
                        .description(format!("i18n:host.method.{key}.desc"))
                        .icon(*icon)
                })
                .collect(),
        );
    let mut children = vec![field(
        "primary_method",
        "i18n:host.method",
        list,
        Some("i18n:host.method.help"),
    )
    .into()];
    if method == PrimaryMethod::Other {
        children.push(
            field(
                "method_other",
                "i18n:host.methodOther",
                text_input(
                    "method_other",
                    config.method_other.host_value(ctx),
                    "i18n:host.methodOther.placeholder",
                ),
                None,
            )
            .visibleWhen("primary_method=other")
            .into(),
        );
    }
    section("method", None, children)
}

// ── 2.2 Codes ────────────────────────────────────────────────────────────────

fn codes_section(method: PrimaryMethod, config: &HostConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = Vec::new();
    // « À chaque séjour » attend un coffre par séjour : pas encore offert.
    let mut scopes = vec![ChoiceOption::new("fixed", "i18n:host.codeScope.fixed")];
    if method == PrimaryMethod::SmartLock {
        scopes.push(ChoiceOption::new("lock", "i18n:host.codeScope.lock"));
    }
    let by_lock = method == PrimaryMethod::SmartLock && config.code_by_lock();
    children.push(
        field(
            "code_scope",
            "i18n:host.codeScope",
            RadioGroup::new()
                .name("code_scope")
                .options(scopes)
                .value(if by_lock { "lock" } else { "fixed" }),
            None,
        )
        .into(),
    );
    match method {
        PrimaryMethod::Keybox => {
            children.push(
                secret_field("keybox_code", "i18n:host.code", &config.keybox_code, None).into(),
            );
            children.push(
                field(
                    "keybox_location",
                    "i18n:host.keybox.location",
                    text_input(
                        "keybox_location",
                        config.keybox_location.host_value(ctx),
                        "i18n:host.keybox.location.placeholder",
                    ),
                    None,
                )
                .into(),
            );
            children.push(
                ToggleRow::new()
                    .name("rotate_reminder")
                    .label("i18n:host.rotate")
                    .description("i18n:host.rotate.help")
                    .checked(config.rotate_reminder)
                    .into(),
            );
        }
        PrimaryMethod::DoorCode => {
            let target = match config.door_code_target.trim() {
                "" => "building",
                target => target,
            };
            children.push(
                field(
                    "door_code_target",
                    "i18n:host.doorCode.target",
                    Select::new()
                        .name("door_code_target")
                        .options(vec![
                            ChoiceOption::new("gate", "i18n:host.doorCode.target.gate"),
                            ChoiceOption::new("building", "i18n:host.doorCode.target.building"),
                            ChoiceOption::new("apartment", "i18n:host.doorCode.target.apartment"),
                        ])
                        .value(target),
                    None,
                )
                .into(),
            );
            children
                .push(secret_field("door_code", "i18n:host.code", &config.door_code, None).into());
        }
        // One stored code: the code itself when it is the same for every stay, the backup code
        // when the lock generates them (§2.2).
        PrimaryMethod::SmartLock => {
            children.push(
                secret_field(
                    "smart_lock_manual_code",
                    "i18n:host.code",
                    &config.smart_lock_manual_code,
                    None,
                )
                .visibleWhen("code_scope=fixed")
                .into(),
            );
            children.push(
                secret_field(
                    "smart_lock_manual_code",
                    "i18n:host.smartLock.manualCode",
                    &config.smart_lock_manual_code,
                    Some("i18n:host.smartLock.manualCode.hint"),
                )
                .visibleWhen("code_scope=lock")
                .into(),
            );
        }
        _ => {}
    }
    section("codes", None, children)
}

// ── 2.3 Révélation des codes ─────────────────────────────────────────────────

fn reveal_section(reveal: RevealPolicy, config: &HostConfig) -> Component {
    let choices = [
        (RevealPolicy::Always, "always"),
        (RevealPolicy::DayBefore16h, "dayBefore16h"),
        (RevealPolicy::HoursBefore24, "hoursBefore24"),
        (RevealPolicy::AtCheckin, "atCheckin"),
        (RevealPolicy::Custom, "custom"),
    ];
    let list = ChoiceList::new()
        .name("reveal_policy")
        .value(reveal.as_wire())
        .emitOnChange(true)
        .layout(ChoiceListLayout::Cards)
        .choices(
            choices
                .iter()
                .map(|(policy, key)| {
                    ChoiceOption::new(policy.as_wire(), format!("i18n:host.reveal.{key}"))
                        .description(format!("i18n:host.reveal.{key}.desc"))
                        .icon(IconName::ClockCircle)
                })
                .collect(),
        );
    let mut children = vec![field(
        "reveal_policy",
        "i18n:config.revealPolicy",
        list,
        Some("i18n:host.reveal.help"),
    )
    .into()];
    if reveal == RevealPolicy::Custom {
        let hours = config.reveal_hours.unwrap_or(DEFAULT_REVEAL_HOURS);
        children.push(
            field(
                "reveal_hours",
                "i18n:host.reveal.hours",
                NumberInput::new()
                    .name("reveal_hours")
                    .value(f64::from(hours))
                    .min(1.0)
                    .max(168.0),
                None,
            )
            .visibleWhen("reveal_policy=custom")
            .into(),
        );
    }
    // Dès la réservation : le code se lit même si le séjour est annulé ensuite (§3).
    if reveal == RevealPolicy::Always {
        children.push(
            InlineNotice::new()
                .message("i18n:publish.reveal.always")
                .tone(Tone::Warning)
                .into(),
        );
    }
    // Masquer après le départ (§2.3) : pas un réglage, une règle — dite ici, en lecture seule.
    children.push(
        KeyValue::new()
            .key("i18n:host.reveal.hideAfter.label")
            .value("i18n:host.reveal.hideAfter.value")
            .into(),
    );
    children.push(FieldHint::new().text("i18n:host.reveal.hideAfter").into());
    section("reveal", None, children)
}

// ── 2.4 Serrure connectée ────────────────────────────────────────────────────

fn lock_section(config: &HostConfig) -> Component {
    let mut children: Vec<Component> = Vec::new();
    push_smart_lock_binding(&mut children, config);
    let window = match config.unlock_window.trim() {
        "reveal" => "reveal",
        _ => "stay",
    };
    children.push(
        field(
            "unlock_window",
            "i18n:host.smartLock.unlockWindow",
            RadioGroup::new()
                .name("unlock_window")
                .options(vec![
                    ChoiceOption::new("stay", "i18n:host.smartLock.unlockWindow.stay"),
                    ChoiceOption::new("reveal", "i18n:host.smartLock.unlockWindow.reveal"),
                ])
                .value(window),
            None,
        )
        .into(),
    );
    section("lock", Some("primary_method=smart_lock"), children)
}

/// The installed smart-lock modules. Their devices are not listed yet: the lock module owns them.
fn push_smart_lock_binding(children: &mut Vec<Component>, config: &HostConfig) {
    let provider = config.smart_lock_provider_module_id.trim();
    let peers = host::module::list_by_capability(portaki_sdk::capability::access::SMART_LOCK)
        .unwrap_or_default();
    let mut options = vec![ChoiceOption::new("", "i18n:host.smartLock.provider.manual")];
    let mut seen = std::collections::BTreeSet::new();
    for peer in &peers {
        if peer.module_id.trim().is_empty() || !seen.insert(peer.module_id.clone()) {
            continue;
        }
        let label = if peer.display_name.trim().is_empty() {
            peer.module_id.as_str().to_string()
        } else {
            peer.display_name.clone()
        };
        options.push(ChoiceOption::new(peer.module_id.as_str(), label));
    }
    // Keep a previously saved id selectable even if not installed yet.
    if !provider.is_empty() && !seen.contains(provider) {
        options.push(ChoiceOption::new(provider, provider));
    }
    children.push(
        field(
            "smart_lock_provider_module_id",
            "i18n:host.smartLock.provider",
            Select::new()
                .name("smart_lock_provider_module_id")
                .options(options)
                .value(provider),
            None,
        )
        .into(),
    );
    let notice = if provider.is_empty() {
        "i18n:host.smartLock.provider.notice.manual"
    } else {
        "i18n:host.smartLock.provider.notice.linked"
    };
    let mut banner = InlineNotice::new().message(notice);
    if !provider.is_empty() && peers.iter().all(|p| p.module_id != provider) {
        banner = banner.tone(Tone::Warning);
    }
    children.push(banner.into());
}

// ── 2.5 Remise des clés ──────────────────────────────────────────────────────

fn handover_section(config: &HostConfig, ctx: &HostContext) -> Component {
    let mut children = vec![field(
        "handover_slot",
        "i18n:host.handover.slot",
        TimeRange::new()
            .name("handover_slot")
            .value(config.handover_slot_raw())
            .min("06:00")
            .max("23:59"),
        Some("i18n:host.handover.slot.hint"),
    )
    .into()];
    children.push(
        field(
            "handover_person",
            "i18n:host.handover.person",
            RadioGroup::new()
                .name("handover_person")
                .options(vec![
                    ChoiceOption::new("me", "i18n:host.handover.person.me"),
                    ChoiceOption::new("other", "i18n:host.handover.person.other"),
                ])
                .value(if config.handover_by_other() {
                    "other"
                } else {
                    "me"
                }),
            None,
        )
        .into(),
    );
    children.push(
        field(
            "handover_name",
            "i18n:host.handover.name",
            text_input(
                "handover_name",
                &config.handover_name,
                "i18n:host.handover.name.placeholder",
            ),
            None,
        )
        .visibleWhen("handover_person=other")
        .into(),
    );
    children.push(
        field(
            "handover_phone",
            "i18n:host.handover.phone",
            phone_input("handover_phone", &config.handover_phone),
            None,
        )
        .visibleWhen("handover_person=other")
        .into(),
    );
    let picker = AddressMapPicker::new()
        .label("i18n:host.inPerson.meetingPlace")
        .hint("i18n:host.inPerson.meetingPlace.hint")
        .addressName("in_person_meeting_place")
        .latName("in_person_meeting_lat")
        .lngName("in_person_meeting_lng")
        .address(config.in_person_meeting_place.host_value(ctx));
    children.push(
        placed(
            picker,
            config.in_person_meeting_lat,
            config.in_person_meeting_lng,
        )
        .into(),
    );
    // Les anciens champs, tant qu'ils portent une valeur : le livret les lit encore, l'hôte
    // doit pouvoir les vider.
    let time_hint = config.in_person_time_hint.host_value(ctx);
    if !time_hint.trim().is_empty() {
        children.push(
            field(
                "in_person_time_hint",
                "i18n:host.inPerson.timeHint",
                TextInput::new()
                    .name("in_person_time_hint")
                    .value(time_hint),
                None,
            )
            .into(),
        );
    }
    if !config.in_person_contact.trim().is_empty() {
        children.push(
            field(
                "in_person_contact",
                "i18n:host.inPerson.contact",
                TextInput::new()
                    .name("in_person_contact")
                    .value(config.in_person_contact.as_str()),
                None,
            )
            .into(),
        );
    }
    section("handover", Some("primary_method=in_person"), children)
}

// ── 2.6 Accueil par le personnel ─────────────────────────────────────────────

fn desk_section(config: &HostConfig, ctx: &HostContext) -> Component {
    let week = match config.desk_hours.trim() {
        "" => DEFAULT_DESK_HOURS,
        week => week,
    };
    let mut children = vec![field(
        "desk_hours",
        "i18n:host.desk.hours",
        WeeklyHours::new()
            .name("desk_hours")
            .value(week)
            .maxRanges(3),
        None,
    )
    .into()];
    children.push(
        field(
            "desk_phone",
            "i18n:host.desk.phone",
            phone_input("desk_phone", &config.desk_phone),
            None,
        )
        .into(),
    );
    children.push(
        field(
            "desk_after_hours",
            "i18n:host.desk.afterHours",
            RichTextEditor::new()
                .name("desk_after_hours")
                .value(config.desk_after_hours.host_value(ctx)),
            None,
        )
        .into(),
    );
    // Les anciens champs, tant qu'ils portent une valeur (voir la remise des clés).
    for (name, label, value) in [
        (
            "building_staff_hours",
            "i18n:host.buildingStaff.hours",
            config.building_staff_hours.host_value(ctx),
        ),
        (
            "building_staff_desk_location",
            "i18n:host.buildingStaff.deskLocation",
            config.building_staff_desk_location.host_value(ctx),
        ),
        (
            "building_staff_contact",
            "i18n:host.buildingStaff.contact",
            config.building_staff_contact.as_str(),
        ),
    ] {
        if !value.trim().is_empty() {
            children
                .push(field(name, label, TextInput::new().name(name).value(value), None).into());
        }
    }
    section("desk", Some("primary_method=building_staff"), children)
}

// ── 2.7 Immeuble ─────────────────────────────────────────────────────────────

fn building_section(enabled: bool, config: &HostConfig, ctx: &HostContext) -> Component {
    // The switch stays: a building switched off kept its values, and they must not reach the
    // guest because the spec dropped it.
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("building_access_enabled")
        .label("i18n:host.building.enabled")
        .checked(enabled)
        .into()];
    if enabled {
        let on = "building_access_enabled=true";
        children.push(
            secret_field(
                "building_access_gate_code",
                "i18n:host.building.gateCode",
                &config.building_access_gate_code,
                None,
            )
            .visibleWhen(on)
            .into(),
        );
        children.push(
            field(
                "building_access_intercom",
                "i18n:host.building.intercom",
                text_input(
                    "building_access_intercom",
                    config.building_access_intercom.host_value(ctx),
                    "i18n:host.building.intercom.placeholder",
                ),
                None,
            )
            .visibleWhen(on)
            .into(),
        );
        children.push(
            field(
                "building_floor",
                "i18n:host.building.floor",
                text_input(
                    "building_floor",
                    config.building_floor.host_value(ctx),
                    "i18n:host.building.floor.placeholder",
                ),
                None,
            )
            .visibleWhen(on)
            .into(),
        );
        let lift = config.building_lift.trim();
        let lift = if LIFT_CHOICES.contains(&lift) {
            lift
        } else {
            "unknown"
        };
        children.push(
            field(
                "building_lift",
                "i18n:host.building.lift",
                RadioGroup::new()
                    .name("building_lift")
                    .options(
                        ["yes", "no", "unknown"]
                            .iter()
                            .map(|o| ChoiceOption::new(*o, format!("i18n:host.building.lift.{o}")))
                            .collect(),
                    )
                    .value(lift),
                None,
            )
            .visibleWhen(on)
            .into(),
        );
        let note = config.building_note.host_value(ctx);
        if !note.trim().is_empty() {
            children.push(
                field(
                    "building_note",
                    "i18n:host.building.note",
                    RichTextEditor::new().name("building_note").value(note),
                    None,
                )
                .into(),
            );
        }
    }
    section("building", None, children)
}

// ── 2.8 Parking ──────────────────────────────────────────────────────────────

/// The card to show selected: `none` without a parking, its kind with one. A parking saved before
/// the kinds has none of them selected — guessing one would tell the guest something false.
fn parking_choice(config: &HostConfig) -> String {
    if !config.parking_on() {
        return "none".into();
    }
    config.parking_kind().unwrap_or_default().to_string()
}

fn parking_section(choice: &str, config: &HostConfig, ctx: &HostContext) -> Component {
    let kinds = std::iter::once(("none", IconName::Ban)).chain(PARKING_KINDS.iter().map(|kind| {
        let icon = match *kind {
            "garage" => IconName::Home,
            "public" => IconName::Parking,
            _ => IconName::Car,
        };
        (*kind, icon)
    }));
    // Loose cards on purpose: they send nothing until clicked, so an old parking without a kind
    // keeps its switch instead of being saved as « Pas de parking ».
    let cards: Vec<Component> = kinds
        .map(|(kind, icon)| {
            SelectableCard::new()
                .name("parking_type")
                .value(kind)
                .label(format!("i18n:host.parking.kind.{kind}"))
                .description(format!("i18n:host.parking.kind.{kind}.desc"))
                .icon(icon)
                .selected(kind == choice)
                .action(emit_input(ParkingTypeInput { parking_type: kind }))
                .into()
        })
        .collect();
    let mut children = vec![field(
        "parking_type",
        "i18n:host.parking.kind",
        Grid::new().columns(3).gap(10.0).children(cards),
        None,
    )
    .into()];
    if choice != "none" {
        children.push(
            field(
                "parking_info",
                "i18n:host.parking.info",
                RichTextEditor::new()
                    .name("parking_info")
                    .value(config.parking_info.host_value(ctx)),
                None,
            )
            .into(),
        );
        children.push(
            field(
                "parking_spot",
                "i18n:host.parking.spot",
                text_input(
                    "parking_spot",
                    &config.parking_spot,
                    "i18n:host.parking.spot.placeholder",
                ),
                None,
            )
            .into(),
        );
        children.push(
            secret_field(
                "parking_code",
                "i18n:host.parking.code",
                &config.parking_code,
                None,
            )
            .into(),
        );
        let picker = AddressMapPicker::new()
            .label("i18n:host.parking.position")
            .hint("i18n:host.parking.position.hint")
            .addressName("parking_address")
            .latName("parking_lat")
            .lngName("parking_lng")
            .address(config.parking_address.as_str());
        children.push(placed(picker, config.parking_lat, config.parking_lng).into());
        if config.parking_too_far(ctx.property.coordinates) {
            children.push(
                InlineNotice::new()
                    .message("i18n:publish.parking.far")
                    .tone(Tone::Warning)
                    .into(),
            );
        }
        // Le tarif, dans la rue ou en parking public seulement.
        if matches!(choice, "street" | "public") {
            children.push(
                field(
                    "parking_price",
                    "i18n:host.parking.price",
                    text_input(
                        "parking_price",
                        config.parking_price.host_value(ctx),
                        "i18n:host.parking.price.placeholder",
                    ),
                    None,
                )
                .into(),
            );
        }
        if !config.parking_map_url.trim().is_empty() {
            children.push(
                field(
                    "parking_map_url",
                    "i18n:host.parking.mapUrl",
                    TextInput::new()
                        .name("parking_map_url")
                        .value(config.parking_map_url.as_str()),
                    None,
                )
                .into(),
            );
        }
    }
    section("parking", None, children)
}

#[derive(Serialize)]
struct ParkingTypeInput<'a> {
    parking_type: &'a str,
}

// ── 2.9 Le chemin jusqu'à la porte ───────────────────────────────────────────

fn path_section(config: &HostConfig, ctx: &HostContext, steps_count: usize) -> Component {
    // The stored rows where they are, blank ones included, then the steps the host adds.
    let rows: Vec<Component> = (0..steps_count)
        .map(|index| step_row(index, ctx, config.steps.get(index)))
        .collect();
    let mut list = StepList::new()
        .label("i18n:host.steps.label")
        .hint("i18n:host.steps.hint")
        .emptyTitle("i18n:host.steps.emptyTitle")
        .emptyDescription("i18n:host.steps.emptyDescription")
        .addLabel("i18n:host.steps.add")
        .removeLabel("i18n:host.steps.remove")
        .itemKeyPrefix("steps")
        .children(rows);
    if steps_count < STEP_SLOTS {
        list = list.addAction(emit_input(StepsCountInput {
            steps_count: steps_count + 1,
        }));
    }
    section("path", None, vec![named("steps").child(list).into()])
}

/// `steps.N.kind`, `.title`, `.detail`, `.photo`, and the row's `.id`. Removing a step blanks
/// every `steps.N.*`, its id included: the platform then clears the row rather than merging.
fn step_row(index: usize, ctx: &HostContext, step: Option<&StepRow>) -> Component {
    let kind = step
        .and_then(|s| s.kind.as_deref())
        .filter(|k| !k.trim().is_empty())
        .unwrap_or("other");
    let title = step.map(|s| s.title.host_value(ctx)).unwrap_or_default();
    let detail = step.map(|s| s.detail.host_value(ctx)).unwrap_or_default();
    let photo = step.map(|s| s.photo.as_str()).unwrap_or_default();
    // A filled row sends its id, so a save merges into it (and keeps its other language). A
    // blank slot has nothing to keep — and an id would make it count as filled.
    let id = step
        .filter(|s| !s.is_blank())
        .map(|s| sdui::row_id("steps", index, Some(&s.id)));
    let name = |key: &str| format!("steps.{index}.{key}");

    Stack::new()
        .id(format!("step-{index}"))
        .gap(10.0)
        .children(
            id.into_iter()
                .chain([
                    field(
                        &name("kind"),
                        "i18n:host.step.kind",
                        Select::new()
                            .name(name("kind"))
                            .options(
                                ["parking", "gate", "door", "elevator", "stairs", "other"]
                                    .iter()
                                    .map(|k| {
                                        ChoiceOption::new(*k, format!("i18n:host.step.kind.{k}"))
                                    })
                                    .collect(),
                            )
                            .value(kind),
                        None,
                    )
                    .into(),
                    field(
                        &name("title"),
                        "i18n:host.step.title",
                        text_input(&name("title"), title, "i18n:host.step.title.placeholder"),
                        None,
                    )
                    .into(),
                    field(
                        &name("detail"),
                        "i18n:host.step.detail",
                        TextInput::new().name(name("detail")).value(detail),
                        None,
                    )
                    .into(),
                    field(
                        &name("photo"),
                        "i18n:host.step.photo",
                        ImageUpload::new().name(name("photo")).value(photo),
                        None,
                    )
                    .into(),
                ])
                .collect(),
        )
        .into()
}

/// The stored rows, all of them, or more once the host adds a step.
fn draft_steps_count(ctx: &HostContext, config: &HostConfig) -> usize {
    let added = ctx.input_u64("steps_count").unwrap_or(0) as usize;
    added.min(STEP_SLOTS).max(config.steps.len())
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct StepsCountInput {
    steps_count: usize,
}

// ── 2.10 À savoir ────────────────────────────────────────────────────────────

fn know_section(config: &HostConfig, ctx: &HostContext) -> Component {
    let mut children: Vec<Component> = vec![
        field(
            "global_note",
            "i18n:host.note.label",
            RichTextEditor::new()
                .name("global_note")
                .value(config.global_note.host_value(ctx)),
            Some("i18n:host.note.help"),
        )
        .into(),
        field(
            "method_instructions",
            "i18n:config.methodInstructions",
            RichTextEditor::new()
                .name("method_instructions")
                .value(config.method_instructions.host_value(ctx)),
            None,
        )
        .into(),
        field(
            "arrival_video_url",
            "i18n:host.video.label",
            TextInput::new()
                .name("arrival_video_url")
                .value(config.arrival_video_url.as_str()),
            None,
        )
        .into(),
        field(
            "late_arrival_note",
            "i18n:host.lateArrival.label",
            RichTextEditor::new()
                .name("late_arrival_note")
                .value(config.late_arrival_note.host_value(ctx)),
            None,
        )
        .into(),
    ];
    // L'adresse vient de Logement › Informations (§2.10) : lue ici, modifiée là-bas. Une
    // adresse saisie ici avant ce changement reste celle que le livret montre.
    let address = match config.address.trim() {
        "" => ctx.property.address.clone().unwrap_or_default(),
        address => address.to_string(),
    };
    if !address.trim().is_empty() {
        children.push(
            KeyValue::new()
                .key("i18n:host.address.label")
                .value(address)
                .into(),
        );
    }
    children.push(
        Link::new()
            .label("i18n:host.address.edit")
            .action(Action::navigate(
                NavigateTarget::path(format!("/listings/{}/general", ctx.property_id)),
                None,
            ))
            .into(),
    );
    section("know", None, children)
}

// ── Draft helpers ────────────────────────────────────────────────────────────

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn parse_primary_method(raw: &str) -> Option<PrimaryMethod> {
    match raw.trim() {
        "host_greets" => Some(PrimaryMethod::InPerson),
        raw => PrimaryMethod::ALL
            .iter()
            .copied()
            .find(|m| m.as_wire() == raw),
    }
}

fn parse_reveal(raw: &str) -> Option<RevealPolicy> {
    RevealPolicy::ALL
        .iter()
        .copied()
        .find(|p| p.as_wire() == raw.trim())
}

/// The saved point back on the map — or no position at all, never 0, 0. Without it, the picker
/// would send blank coordinates and each save would wipe them.
fn placed(picker: AddressMapPicker, lat: Option<f64>, lng: Option<f64>) -> AddressMapPicker {
    match coord_pair(lat, lng) {
        Some((lat, lng)) => picker.lat(lat).lng(lng),
        None => picker,
    }
}

// ── Field helpers ────────────────────────────────────────────────────────────

thread_local! {
    /// Les erreurs du rendu en cours, par nom de champ.
    ///
    /// ponytail: un état de rendu plutôt qu'un paramètre de plus à chacune des aides de champ,
    /// appelées de vingt endroits. Le Wasm d'un module rend une surface à la fois ; à remplacer
    /// par un paramètre si le rendu devient concurrent.
    static ERRORS: std::cell::RefCell<Vec<(String, String)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Le champ `name`, avec son message d'erreur du rendu en cours s'il y en a un.
fn named(name: &str) -> Field {
    let field = Field::new().name(name);
    let error = ERRORS.with(|errors| {
        errors
            .borrow()
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, error)| error.clone())
    });
    match error {
        Some(error) => field.error(error),
        None => field,
    }
}

/// Libellé, contrôle, et l'aide dessous — que l'erreur remplace (§2.3 des règles communes).
fn field(name: &str, label: &str, control: impl Into<Component>, hint: Option<&str>) -> Field {
    let field = named(name).label(label).child(control);
    match hint {
        Some(hint) => field.child(FieldHint::new().text(hint)),
        None => field,
    }
}

fn text_input(name: &str, value: &str, placeholder: &str) -> TextInput {
    TextInput::new()
        .name(name)
        .value(value)
        .placeholder(placeholder)
}

/// E.164 : un numéro saisi avec des espaces avant ce champ revient composable.
fn phone_input(name: &str, value: &str) -> PhoneInput {
    PhoneInput::new()
        .name(name)
        .value(crate::config::dialable(value))
}

/// A code is never sent back to the form: blank keeps it (the platform ignores `""`).
fn secret_field(name: &str, label: &str, saved: &str, hint: Option<&str>) -> Field {
    let mut input = SecretInput::new().name(name).value(String::new());
    if !saved.trim().is_empty() {
        input = input.placeholder("i18n:host.secret.keep");
    }
    field(name, label, input, hint)
}
