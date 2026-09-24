//! Host dashboard surfaces — conditional access configuration form (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Button, Card, ChoiceList, Field, FieldHint, Form, Grid, InlineNotice, Page,
    RichTextEditor, SecretInput, Select, Stack, StepList, Text, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{
    load_config, AccessStep, DoorCodeTarget, MethodFields, ModuleConfig, PrimaryMethod,
    RevealPolicy, StaffKind,
};
use crate::texts::{load_texts_for_host, ModuleTexts};

const STEP_SLOTS: usize = 8;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::AccessEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Key
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let texts = load_texts_for_host(&ctx.locale).unwrap_or_default();
    let draft_method = draft_primary_method(&ctx, &config);
    let building_enabled = ctx.input_bool(
        "building_access_enabled",
        config.building_access.is_some() || texts.building_note.is_some(),
    );
    let parking_enabled = ctx.input_bool(
        "parking_enabled",
        config.parking.is_some() || !texts.parking_info.trim().is_empty(),
    );
    let steps_count = draft_steps_count(&ctx, &config);

    let submit_args = crate::commands::UpdateConfigArgs {
        primary_method: Some(draft_method),
        building_access_enabled: Some(building_enabled),
        parking_enabled: Some(parking_enabled),
        reveal_policy: Some(config.reveal_policy),
        ..Default::default()
    };
    let save_action = crate::ids::module_id().command(crate::ids::UPDATE_CONFIG, submit_args);

    // Reveal timing only applies when there is (or can be) a code: primary
    // method with credential, and/or optional building / parking layers.
    let show_reveal = draft_method.involves_access_code() || building_enabled || parking_enabled;

    let mut form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.primary")
            .subtitle("i18n:host.section.primary.help")
            .icon(IconName::Key)
            .children(vec![method_choice_list(draft_method).into()])
            .into(),
        Card::new()
            .title("i18n:host.section.methodDetails")
            .subtitle("i18n:host.section.methodDetails.help")
            .icon(IconName::Lock)
            .children(method_detail_children(draft_method, &config, &texts))
            .into(),
        Grid::new()
            .columns(2)
            .gap(16.0)
            .children(vec![
                layer_card_building(building_enabled, &config, &texts),
                layer_card_parking(parking_enabled, &config, &texts),
            ])
            .into(),
        Card::new()
            .title("i18n:host.section.arrival")
            .subtitle("i18n:host.section.arrival.help")
            .icon(IconName::MapPin)
            .children(arrival_children(&config, &texts, steps_count))
            .into(),
    ];

    if show_reveal {
        form_children.push(
            Card::new()
                .title("i18n:host.section.reveal")
                .subtitle("i18n:host.section.reveal.help")
                .icon(IconName::ClockCircle)
                .children(vec![reveal_choice_list(config.reveal_policy).into()])
                .into(),
        );
    }

    form_children.push(
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
    );
    form_children.push(
        Button::new()
            .label("i18n:host.save")
            .action(save_action)
            .tone(Tone::Primary)
            .into(),
    );

    Surface::new(Page::new().child(Form::new().children(form_children)))
        .with_id(crate::ids::HOST_MAIN)
}

// ── Draft helpers ────────────────────────────────────────────────────────────

fn draft_primary_method(ctx: &HostContext, config: &ModuleConfig) -> PrimaryMethod {
    ctx.input_str("primary_method")
        .and_then(parse_primary_method)
        .unwrap_or(config.primary_method)
}

fn draft_steps_count(ctx: &HostContext, config: &ModuleConfig) -> usize {
    if let Some(n) = ctx.input_u64("steps_count") {
        return (n as usize).min(STEP_SLOTS);
    }
    let existing = config.parse_steps().len();
    existing.min(STEP_SLOTS)
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct StepsCountInput {
    steps_count: usize,
}

fn emit_input(payload: impl Serialize) -> Action {
    Action::emit(contracts::shell::SURFACE_INPUT, Some(json_value(payload)))
}

fn parse_primary_method(raw: &str) -> Option<PrimaryMethod> {
    PrimaryMethod::ALL
        .iter()
        .copied()
        .find(|m| m.as_wire() == raw.trim())
}

fn door_target_str(target: DoorCodeTarget) -> &'static str {
    match target {
        DoorCodeTarget::Gate => "gate",
        DoorCodeTarget::Building => "building",
        DoorCodeTarget::Apartment => "apartment",
    }
}

fn staff_kind_str(kind: StaffKind) -> &'static str {
    match kind {
        StaffKind::Reception => "reception",
        StaffKind::Caretaker => "caretaker",
    }
}

// ── Choice lists ─────────────────────────────────────────────────────────────

fn method_choice_list(selected: PrimaryMethod) -> ChoiceList {
    ChoiceList::new()
        .name("primary_method")
        .value(selected.as_wire())
        .emitOnChange(true)
        .layout(ChoiceListLayout::Cards)
        .choices(vec![
            ChoiceOption::new(PrimaryMethod::Keybox.as_wire(), "i18n:host.method.keybox")
                .description("i18n:host.method.keybox.desc")
                .icon(IconName::Key),
            ChoiceOption::new(
                PrimaryMethod::DoorCode.as_wire(),
                "i18n:host.method.door_code",
            )
            .description("i18n:host.method.door_code.desc")
            .icon(IconName::Grid),
            ChoiceOption::new(
                PrimaryMethod::SmartLock.as_wire(),
                "i18n:host.method.smart_lock",
            )
            .description("i18n:host.method.smart_lock.desc")
            .icon(IconName::Lock),
            ChoiceOption::new(
                PrimaryMethod::InPerson.as_wire(),
                "i18n:host.method.in_person",
            )
            .description("i18n:host.method.in_person.desc")
            .icon(IconName::Users),
            ChoiceOption::new(
                PrimaryMethod::BuildingStaff.as_wire(),
                "i18n:host.method.building_staff",
            )
            .description("i18n:host.method.building_staff.desc")
            .icon(IconName::Building),
            ChoiceOption::new(
                PrimaryMethod::HostGreets.as_wire(),
                "i18n:host.method.host_greets",
            )
            .description("i18n:host.method.host_greets.desc")
            .icon(IconName::Smile),
            ChoiceOption::new(PrimaryMethod::Other.as_wire(), "i18n:host.method.other")
                .description("i18n:host.method.other.desc")
                .icon(IconName::MoreHorizontal),
        ])
}

fn reveal_choice_list(policy: RevealPolicy) -> ChoiceList {
    ChoiceList::new()
        .name("reveal_policy")
        .value(policy.as_wire())
        .layout(ChoiceListLayout::Compact)
        .choices(vec![
            ChoiceOption::new(RevealPolicy::Always.as_wire(), "i18n:host.reveal.always")
                .description("i18n:host.reveal.always.desc")
                .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::HoursBefore24.as_wire(),
                "i18n:host.reveal.hoursBefore24",
            )
            .description("i18n:host.reveal.hoursBefore24.desc")
            .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::DayBefore16h.as_wire(),
                "i18n:host.reveal.dayBefore16h",
            )
            .description("i18n:host.reveal.dayBefore16h.desc")
            .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::AtCheckin.as_wire(),
                "i18n:host.reveal.atCheckin",
            )
            .description("i18n:host.reveal.atCheckin.desc")
            .icon(IconName::ClockCircle),
        ])
}

// ── Method details ───────────────────────────────────────────────────────────

fn method_detail_children(
    method: PrimaryMethod,
    config: &ModuleConfig,
    texts: &ModuleTexts,
) -> Vec<Component> {
    let mut children = Vec::new();
    match method {
        PrimaryMethod::Keybox => push_keybox_fields(&mut children, config, texts),
        PrimaryMethod::DoorCode => push_door_code_fields(&mut children, config, texts),
        PrimaryMethod::SmartLock => {
            push_smart_lock_binding(&mut children, config);
            push_smart_lock_fields(&mut children, config, texts);
        }
        PrimaryMethod::InPerson => push_in_person_fields(&mut children, config),
        PrimaryMethod::BuildingStaff => push_building_staff_fields(&mut children, config),
        PrimaryMethod::HostGreets => push_host_greets_fields(&mut children, config),
        PrimaryMethod::Other => push_other_fields(&mut children, texts),
    }
    children
}

fn method_instructions(texts: &ModuleTexts) -> &str {
    texts.method_instructions.as_deref().unwrap_or("")
}

fn push_keybox_fields(children: &mut Vec<Component>, config: &ModuleConfig, texts: &ModuleTexts) {
    let (location, code) = match &config.method {
        MethodFields::Keybox { location, code } => {
            (location.as_str(), code.as_deref().unwrap_or(""))
        }
        _ => ("", ""),
    };
    children.push(text_field(
        "keybox_location",
        "i18n:host.keybox.location",
        location,
    ));
    children.push(
        FieldHint::new()
            .text("i18n:host.keybox.location.hint")
            .into(),
    );
    children.push(secret_field("keybox_code", "i18n:host.keybox.code", code));
    children.push(FieldHint::new().text("i18n:host.keybox.code.hint").into());
    children.push(rich_text_field(
        "keybox_instructions",
        "i18n:host.keybox.instructions",
        method_instructions(texts),
    ));
}

fn push_door_code_fields(
    children: &mut Vec<Component>,
    config: &ModuleConfig,
    texts: &ModuleTexts,
) {
    let (target, code) = match &config.method {
        MethodFields::DoorCode { target, code } => (*target, code.as_str()),
        _ => (DoorCodeTarget::Building, ""),
    };
    children.push(
        Field::new()
            .name("door_code_target")
            .label("i18n:host.doorCode.target")
            .child(
                Select::new()
                    .name("door_code_target")
                    .options(vec![
                        ChoiceOption::new("gate", "i18n:host.doorCode.target.gate"),
                        ChoiceOption::new("building", "i18n:host.doorCode.target.building"),
                        ChoiceOption::new("apartment", "i18n:host.doorCode.target.apartment"),
                    ])
                    .value(door_target_str(target)),
            )
            .into(),
    );
    children.push(
        FieldHint::new()
            .text("i18n:host.doorCode.target.hint")
            .into(),
    );
    children.push(secret_field("door_code", "i18n:host.doorCode.code", code));
    children.push(FieldHint::new().text("i18n:host.doorCode.code.hint").into());
    children.push(rich_text_field(
        "door_code_instructions",
        "i18n:host.doorCode.instructions",
        method_instructions(texts),
    ));
}

fn push_smart_lock_fields(
    children: &mut Vec<Component>,
    config: &ModuleConfig,
    texts: &ModuleTexts,
) {
    let manual_code = match &config.method {
        MethodFields::SmartLock { manual_code } => manual_code.as_deref().unwrap_or(""),
        _ => "",
    };
    children.push(secret_field(
        "smart_lock_manual_code",
        "i18n:host.smartLock.manualCode",
        manual_code,
    ));
    children.push(
        FieldHint::new()
            .text("i18n:host.smartLock.manualCode.hint")
            .into(),
    );
    children.push(rich_text_field(
        "smart_lock_instructions",
        "i18n:host.smartLock.instructions",
        method_instructions(texts),
    ));
}

fn push_smart_lock_binding(children: &mut Vec<Component>, config: &ModuleConfig) {
    let provider = config
        .smart_lock_provider_module_id
        .as_deref()
        .unwrap_or("");
    let peers = host::module::list_by_capability(portaki_sdk::capability::access::SMART_LOCK)
        .unwrap_or_default();
    children.push(
        Field::new()
            .name("smart_lock_provider_module_id")
            .label("i18n:host.smartLock.provider")
            .child(
                Select::new()
                    .name("smart_lock_provider_module_id")
                    .options({
                        let mut opts =
                            vec![ChoiceOption::new("", "i18n:host.smartLock.provider.manual")];
                        let mut seen = std::collections::BTreeSet::new();
                        for peer in &peers {
                            if peer.module_id.trim().is_empty()
                                || !seen.insert(peer.module_id.clone())
                            {
                                continue;
                            }
                            let label = if peer.display_name.trim().is_empty() {
                                peer.module_id.as_str().to_string()
                            } else {
                                peer.display_name.clone()
                            };
                            opts.push(ChoiceOption::new(peer.module_id.as_str(), label));
                        }
                        // Keep a previously saved id selectable even if not installed yet.
                        if !provider.is_empty() && !seen.contains(provider) {
                            opts.push(ChoiceOption::new(provider, provider));
                        }
                        opts
                    })
                    .value(provider),
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

fn push_in_person_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let (place, lat, lng, time_hint, contact) = match &config.method {
        MethodFields::InPerson {
            meeting_place,
            lat,
            lng,
            time_hint,
            contact,
        } => (
            meeting_place.as_str(),
            *lat,
            *lng,
            time_hint.as_deref().unwrap_or(""),
            contact.as_deref().unwrap_or(""),
        ),
        _ => ("", None, None, "", ""),
    };
    children.push(
        AddressMapPicker::new()
            .label("i18n:host.inPerson.meetingPlace")
            .hint("i18n:host.inPerson.meetingPlace.hint")
            .addressName("in_person_meeting_place")
            .latName("in_person_meeting_lat")
            .lngName("in_person_meeting_lng")
            .address(place)
            .lat(lat.unwrap_or(0.0))
            .lng(lng.unwrap_or(0.0))
            .into(),
    );
    children.push(text_field(
        "in_person_time_hint",
        "i18n:host.inPerson.timeHint",
        time_hint,
    ));
    children.push(text_field(
        "in_person_contact",
        "i18n:host.inPerson.contact",
        contact,
    ));
}

fn push_building_staff_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let (kind, desk, hours, contact) = match &config.method {
        MethodFields::BuildingStaff {
            staff_kind,
            desk_location,
            hours,
            contact,
        } => (
            *staff_kind,
            desk_location.as_str(),
            hours.as_deref().unwrap_or(""),
            contact.as_deref().unwrap_or(""),
        ),
        _ => (StaffKind::Reception, "", "", ""),
    };
    children.push(
        Field::new()
            .name("building_staff_kind")
            .label("i18n:host.buildingStaff.kind")
            .child(
                Select::new()
                    .name("building_staff_kind")
                    .options(vec![
                        ChoiceOption::new("reception", "i18n:host.buildingStaff.kind.reception"),
                        ChoiceOption::new("caretaker", "i18n:host.buildingStaff.kind.caretaker"),
                    ])
                    .value(staff_kind_str(kind)),
            )
            .into(),
    );
    children.push(text_field(
        "building_staff_desk_location",
        "i18n:host.buildingStaff.deskLocation",
        desk,
    ));
    children.push(text_field(
        "building_staff_hours",
        "i18n:host.buildingStaff.hours",
        hours,
    ));
    children.push(text_field(
        "building_staff_contact",
        "i18n:host.buildingStaff.contact",
        contact,
    ));
}

fn push_host_greets_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let (note, eta) = match &config.method {
        MethodFields::HostGreets {
            contact_note,
            eta_hint,
        } => (
            contact_note.as_deref().unwrap_or(""),
            eta_hint.as_deref().unwrap_or(""),
        ),
        _ => ("", ""),
    };
    children.push(rich_text_field(
        "host_greets_contact_note",
        "i18n:host.hostGreets.contactNote",
        note,
    ));
    children.push(text_field(
        "host_greets_eta_hint",
        "i18n:host.hostGreets.etaHint",
        eta,
    ));
}

fn push_other_fields(children: &mut Vec<Component>, texts: &ModuleTexts) {
    children.push(rich_text_field(
        "other_instructions",
        "i18n:host.other.instructions",
        method_instructions(texts),
    ));
}

// ── Layers ───────────────────────────────────────────────────────────────────

fn layer_card_building(enabled: bool, config: &ModuleConfig, texts: &ModuleTexts) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("building_access_enabled")
        .label("i18n:host.building.enabled")
        .checked(enabled)
        .into()];
    if enabled {
        let gate = config
            .building_access
            .as_ref()
            .and_then(|b| b.gate_code.as_deref())
            .unwrap_or("");
        let intercom = config
            .building_access
            .as_ref()
            .and_then(|b| b.intercom.as_deref())
            .unwrap_or("");
        let note = texts.building_note.as_deref().unwrap_or("");
        children.push(secret_field(
            "building_access_gate_code",
            "i18n:host.building.gateCode",
            gate,
        ));
        children.push(text_field(
            "building_access_intercom",
            "i18n:host.building.intercom",
            intercom,
        ));
        children.push(rich_text_field(
            "building_access_note",
            "i18n:host.building.note",
            note,
        ));
    } else {
        children.push(
            Text::new()
                .text("i18n:host.layer.disabled")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    Card::new()
        .title("i18n:host.section.building")
        .subtitle("i18n:host.section.building.help")
        .icon(IconName::Building)
        .children(children)
        .into()
}

fn layer_card_parking(enabled: bool, config: &ModuleConfig, texts: &ModuleTexts) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("parking_enabled")
        .label("i18n:host.parking.enabled")
        .checked(enabled)
        .into()];
    if enabled {
        let info = texts.parking_info.as_str();
        let map_url = config
            .parking
            .as_ref()
            .map(|p| p.map_url.as_str())
            .unwrap_or("");
        let code = config
            .parking
            .as_ref()
            .and_then(|p| p.code.as_deref())
            .unwrap_or("");
        children.push(rich_text_field(
            "parking_info",
            "i18n:host.parking.info",
            info,
        ));
        children.push(text_field(
            "parking_map_url",
            "i18n:host.parking.mapUrl",
            map_url,
        ));
        children.push(secret_field("parking_code", "i18n:host.parking.code", code));
    } else {
        children.push(
            Text::new()
                .text("i18n:host.layer.disabled")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    Card::new()
        .title("i18n:host.section.parking")
        .subtitle("i18n:host.section.parking.help")
        .icon(IconName::Car)
        .children(children)
        .into()
}

// ── Arrival ──────────────────────────────────────────────────────────────────

fn arrival_children(
    config: &ModuleConfig,
    texts: &ModuleTexts,
    steps_count: usize,
) -> Vec<Component> {
    let mut children: Vec<Component> = Vec::new();
    children.push(
        AddressMapPicker::new()
            .label("i18n:host.address.label")
            .hint("i18n:host.address.hint")
            .addressName("address")
            .latName("arrival_lat")
            .lngName("arrival_lng")
            .address(config.arrival.address.as_str())
            .into(),
    );

    let steps = config.parse_steps();
    let mut step_rows: Vec<Component> = Vec::new();
    for index in 0..steps_count {
        let skeleton = steps.get(index);
        let text = skeleton.and_then(|s| texts.step_by_id(&s.id));
        step_rows.push(step_row(index, skeleton, text));
    }

    children.push(
        StepList::new()
            .label("i18n:host.steps.label")
            .hint("i18n:host.steps.hint")
            .emptyTitle("i18n:host.steps.emptyTitle")
            .emptyDescription("i18n:host.steps.emptyDescription")
            .addLabel("i18n:host.steps.add")
            .removeLabel("i18n:host.steps.remove")
            .itemKeyPrefix("steps")
            .addAction(emit_input(StepsCountInput {
                steps_count: (steps_count + 1).min(STEP_SLOTS),
            }))
            .children(step_rows)
            .into(),
    );

    children.push(text_field(
        "arrival_video_url",
        "i18n:host.video.label",
        &config.arrival.arrival_video_url,
    ));
    children.push(rich_text_field(
        "global_note",
        "i18n:host.note.label",
        &texts.global_note,
    ));
    children
}

fn step_row(
    index: usize,
    step: Option<&AccessStep>,
    text: Option<&crate::texts::StepText>,
) -> Component {
    let kind = step.and_then(|s| s.kind.as_deref()).unwrap_or("other");
    let title = text.map(|t| t.title.as_str()).unwrap_or("");
    let detail = text.and_then(|t| t.detail.as_deref()).unwrap_or("");

    Stack::new()
        .id(format!("step-{index}"))
        .gap(10.0)
        .children(vec![
            Field::new()
                .name(format!("steps.{index}.kind"))
                .label("i18n:host.step.kind")
                .child(
                    Select::new()
                        .name(format!("steps.{index}.kind"))
                        .options(vec![
                            ChoiceOption::new("parking", "i18n:host.step.kind.parking"),
                            ChoiceOption::new("door", "i18n:host.step.kind.door"),
                            ChoiceOption::new("elevator", "i18n:host.step.kind.elevator"),
                            ChoiceOption::new("other", "i18n:host.step.kind.other"),
                        ])
                        .value(kind),
                )
                .into(),
            text_field(
                &format!("steps.{index}.title"),
                "i18n:host.step.title",
                title,
            ),
            text_field(
                &format!("steps.{index}.detail"),
                "i18n:host.step.detail",
                detail,
            ),
        ])
        .into()
}

// ── Field helpers ────────────────────────────────────────────────────────────

fn text_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(name)
        .label(label_key)
        .child(TextInput::new().name(name).value(value))
        .into()
}

fn secret_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(name)
        .label(label_key)
        .child(SecretInput::new().name(name).value(value))
        .into()
}

fn rich_text_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(name)
        .label(label_key)
        .child(RichTextEditor::new().name(name).value(value))
        .into()
}
