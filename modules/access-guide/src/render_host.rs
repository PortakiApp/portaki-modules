//! Host dashboard surfaces — conditional access configuration form.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Button, Field, Form, Page, Select, Stack, Text, TextArea, TextInput,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Value};

use crate::config::{
    load_config, AccessStep, DoorCodeTarget, MethodFields, ModuleConfig, PrimaryMethod,
    RevealPolicy, StaffKind,
};

const STEP_SLOTS: usize = 6;
const EMIT_SURFACE_INPUT: &str = "host.surface.input";

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let draft_method = draft_primary_method(&ctx.input, &config);
    let building_enabled = draft_flag(
        &ctx.input,
        "building_access_enabled",
        config.building_access.is_some(),
    );
    let parking_enabled = draft_flag(&ctx.input, "parking_enabled", config.parking.is_some());

    let submit_args = json!({
        "primary_method": primary_method_str(draft_method),
        "building_access_enabled": building_enabled,
        "parking_enabled": parking_enabled,
        "reveal_policy": reveal_policy_str(config.reveal_policy),
    });
    let save_action =
        serde_json::to_value(Action::command("access-guide", "updateConfig", submit_args))
            .unwrap_or(json!({}));

    let mut form_children: Vec<Component> = Vec::new();

    push_section(
        &mut form_children,
        "i18n:host.section.primary",
        "i18n:host.section.primary.help",
    );
    push_method_picker(&mut form_children, draft_method);
    push_method_fields(&mut form_children, draft_method, &config);

    if draft_method == PrimaryMethod::SmartLock {
        push_smart_lock_binding(&mut form_children, &config);
    }

    push_section(
        &mut form_children,
        "i18n:host.section.building",
        "i18n:host.section.building.help",
    );
    push_layer_toggle(
        &mut form_children,
        "building_access_enabled",
        building_enabled,
        "i18n:host.building.enabled",
    );
    if building_enabled {
        push_building_fields(&mut form_children, &config);
    }

    push_section(
        &mut form_children,
        "i18n:host.section.parking",
        "i18n:host.section.parking.help",
    );
    push_layer_toggle(
        &mut form_children,
        "parking_enabled",
        parking_enabled,
        "i18n:host.parking.enabled",
    );
    if parking_enabled {
        push_parking_fields(&mut form_children, &config);
    }

    push_section(
        &mut form_children,
        "i18n:host.section.arrival",
        "i18n:host.section.arrival.help",
    );
    push_arrival_fields(&mut form_children, &config);

    push_section(
        &mut form_children,
        "i18n:host.section.reveal",
        "i18n:host.section.reveal.help",
    );
    push_reveal_policy(&mut form_children, config.reveal_policy);

    form_children.push(
        Text::new()
            .text(json!("i18n:host.main.help"))
            .variant(json!("caption"))
            .into(),
    );
    form_children.push(
        Button::new()
            .label(json!("i18n:host.save"))
            .action(save_action)
            .tone(Tone::Primary)
            .into(),
    );

    Surface::new(
        Page::new()
            .title(json!("i18n:surface.host.main.title"))
            .child(
                Text::new()
                    .text(json!("i18n:surface.host.main.subtitle"))
                    .variant(json!("body")),
            )
            .child(Form::new().children(form_children)),
    )
    .with_id("main")
}

// ── Draft helpers ────────────────────────────────────────────────────────────

fn draft_primary_method(input: &Value, config: &ModuleConfig) -> PrimaryMethod {
    input
        .get("primary_method")
        .and_then(|v| v.as_str())
        .and_then(parse_primary_method)
        .unwrap_or(config.primary_method)
}

fn draft_flag(input: &Value, key: &str, fallback: bool) -> bool {
    match input.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) if s == "true" => true,
        Some(Value::String(s)) if s == "false" => false,
        _ => fallback,
    }
}

fn emit_input(payload: Value) -> Value {
    serde_json::to_value(Action::Emit {
        event: EMIT_SURFACE_INPUT.into(),
        payload: Some(payload),
    })
    .unwrap_or(json!({}))
}

fn primary_method_str(method: PrimaryMethod) -> &'static str {
    match method {
        PrimaryMethod::Keybox => "keybox",
        PrimaryMethod::DoorCode => "door_code",
        PrimaryMethod::SmartLock => "smart_lock",
        PrimaryMethod::InPerson => "in_person",
        PrimaryMethod::BuildingStaff => "building_staff",
        PrimaryMethod::HostGreets => "host_greets",
        PrimaryMethod::Other => "other",
    }
}

fn parse_primary_method(raw: &str) -> Option<PrimaryMethod> {
    match raw.trim() {
        "keybox" => Some(PrimaryMethod::Keybox),
        "door_code" => Some(PrimaryMethod::DoorCode),
        "smart_lock" => Some(PrimaryMethod::SmartLock),
        "in_person" => Some(PrimaryMethod::InPerson),
        "building_staff" => Some(PrimaryMethod::BuildingStaff),
        "host_greets" => Some(PrimaryMethod::HostGreets),
        "other" => Some(PrimaryMethod::Other),
        _ => None,
    }
}

fn reveal_policy_str(policy: RevealPolicy) -> &'static str {
    match policy {
        RevealPolicy::Always => "always",
        RevealPolicy::HoursBefore24 => "hours_before_24",
        RevealPolicy::DayBefore16h => "day_before_16h",
        RevealPolicy::AtCheckin => "at_checkin",
    }
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

// ── Sections ─────────────────────────────────────────────────────────────────

fn push_section(children: &mut Vec<Component>, title_key: &str, help_key: &str) {
    children.push(
        Text::new()
            .text(json!(title_key))
            .variant(json!("title"))
            .into(),
    );
    children.push(
        Text::new()
            .text(json!(help_key))
            .variant(json!("caption"))
            .into(),
    );
}

fn push_method_picker(children: &mut Vec<Component>, selected: PrimaryMethod) {
    let options: [(PrimaryMethod, &str); 7] = [
        (PrimaryMethod::Keybox, "i18n:host.method.keybox"),
        (PrimaryMethod::DoorCode, "i18n:host.method.door_code"),
        (PrimaryMethod::SmartLock, "i18n:host.method.smart_lock"),
        (PrimaryMethod::InPerson, "i18n:host.method.in_person"),
        (
            PrimaryMethod::BuildingStaff,
            "i18n:host.method.building_staff",
        ),
        (PrimaryMethod::HostGreets, "i18n:host.method.host_greets"),
        (PrimaryMethod::Other, "i18n:host.method.other"),
    ];

    let buttons: Vec<Component> = options
        .into_iter()
        .map(|(method, label)| {
            let mut button = Button::new().label(json!(label)).action(emit_input(
                json!({ "primary_method": primary_method_str(method) }),
            ));
            if method == selected {
                button = button.tone(Tone::Primary);
            }
            button.into()
        })
        .collect();

    children.push(
        Stack::new()
            .direction(json!("horizontal"))
            .gap(json!(8))
            .children(buttons)
            .into(),
    );
}

fn push_layer_toggle(children: &mut Vec<Component>, key: &str, enabled: bool, label_key: &str) {
    let mut on = Button::new()
        .label(json!(label_key))
        .action(emit_input(json!({ key: true })));
    let mut off = Button::new()
        .label(json!("i18n:host.layer.disable"))
        .action(emit_input(json!({ key: false })));
    if enabled {
        on = on.tone(Tone::Primary);
    } else {
        off = off.tone(Tone::Primary);
    }
    children.push(
        Stack::new()
            .direction(json!("horizontal"))
            .gap(json!(8))
            .children(vec![on.into(), off.into()])
            .into(),
    );
}

// ── Method fields ────────────────────────────────────────────────────────────

fn push_method_fields(children: &mut Vec<Component>, method: PrimaryMethod, config: &ModuleConfig) {
    match method {
        PrimaryMethod::Keybox => push_keybox_fields(children, config),
        PrimaryMethod::DoorCode => push_door_code_fields(children, config),
        PrimaryMethod::SmartLock => push_smart_lock_fields(children, config),
        PrimaryMethod::InPerson => push_in_person_fields(children, config),
        PrimaryMethod::BuildingStaff => push_building_staff_fields(children, config),
        PrimaryMethod::HostGreets => push_host_greets_fields(children, config),
        PrimaryMethod::Other => push_other_fields(children, config),
    }
}

fn push_keybox_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let (location, code, instructions) = match &config.method {
        MethodFields::Keybox {
            location,
            code,
            instructions,
        } => (
            location.as_str(),
            code.as_deref().unwrap_or(""),
            instructions.as_deref().unwrap_or(""),
        ),
        _ => ("", "", ""),
    };
    children.push(text_field(
        "keybox_location",
        "i18n:host.keybox.location",
        location,
    ));
    children.push(text_field("keybox_code", "i18n:host.keybox.code", code));
    children.push(text_area_field(
        "keybox_instructions",
        "i18n:host.keybox.instructions",
        instructions,
    ));
}

fn push_door_code_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let (target, code, instructions) = match &config.method {
        MethodFields::DoorCode {
            target,
            code,
            instructions,
        } => (
            *target,
            code.as_str(),
            instructions.as_deref().unwrap_or(""),
        ),
        _ => (DoorCodeTarget::Building, "", ""),
    };
    children.push(
        Field::new()
            .name(json!("door_code_target"))
            .label(json!("i18n:host.doorCode.target"))
            .child(
                Select::new()
                    .name(json!("door_code_target"))
                    .options(json!([
                        {"value": "gate", "label": "i18n:host.doorCode.target.gate"},
                        {"value": "building", "label": "i18n:host.doorCode.target.building"},
                        {"value": "apartment", "label": "i18n:host.doorCode.target.apartment"}
                    ]))
                    .value(json!(door_target_str(target))),
            )
            .into(),
    );
    children.push(text_field("door_code", "i18n:host.doorCode.code", code));
    children.push(text_area_field(
        "door_code_instructions",
        "i18n:host.doorCode.instructions",
        instructions,
    ));
}

fn push_smart_lock_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let (instructions, manual_code) = match &config.method {
        MethodFields::SmartLock {
            instructions,
            manual_code,
        } => (
            instructions.as_deref().unwrap_or(""),
            manual_code.as_deref().unwrap_or(""),
        ),
        _ => ("", ""),
    };
    children.push(text_area_field(
        "smart_lock_instructions",
        "i18n:host.smartLock.instructions",
        instructions,
    ));
    children.push(text_field(
        "smart_lock_manual_code",
        "i18n:host.smartLock.manualCode",
        manual_code,
    ));
}

fn push_smart_lock_binding(children: &mut Vec<Component>, config: &ModuleConfig) {
    let provider = config
        .smart_lock_provider_module_id
        .as_deref()
        .unwrap_or("");
    children.push(
        Text::new()
            .text(json!("i18n:host.smartLock.provider.help"))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!("smart_lock_provider_module_id"))
            .label(json!("i18n:host.smartLock.provider"))
            .child(
                Select::new()
                    .name(json!("smart_lock_provider_module_id"))
                    .options({
                        let mut opts = vec![json!({
                            "value": "",
                            "label": "i18n:host.smartLock.provider.manual"
                        })];
                        if !provider.is_empty() {
                            opts.push(json!({
                                "value": provider,
                                "label": provider
                            }));
                        }
                        Value::Array(opts)
                    })
                    .value(json!(provider)),
            )
            .into(),
    );
    children.push(text_field(
        "smart_lock_provider_module_id_custom",
        "i18n:host.smartLock.provider.custom",
        "",
    ));
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
    children.push(text_field(
        "in_person_meeting_place",
        "i18n:host.inPerson.meetingPlace",
        place,
    ));
    children.push(text_field(
        "in_person_meeting_lat",
        "i18n:host.inPerson.lat",
        &format_coord(lat),
    ));
    children.push(text_field(
        "in_person_meeting_lng",
        "i18n:host.inPerson.lng",
        &format_coord(lng),
    ));
    children.push(
        Text::new()
            .text(json!("i18n:host.inPerson.coordsHelp"))
            .variant(json!("caption"))
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

fn format_coord(value: Option<f64>) -> String {
    value.map(|v| format!("{v:.6}")).unwrap_or_default()
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
            .name(json!("building_staff_kind"))
            .label(json!("i18n:host.buildingStaff.kind"))
            .child(
                Select::new()
                    .name(json!("building_staff_kind"))
                    .options(json!([
                        {"value": "reception", "label": "i18n:host.buildingStaff.kind.reception"},
                        {"value": "caretaker", "label": "i18n:host.buildingStaff.kind.caretaker"}
                    ]))
                    .value(json!(staff_kind_str(kind))),
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
    children.push(text_area_field(
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

fn push_other_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let instructions = match &config.method {
        MethodFields::Other { instructions } => instructions.as_str(),
        _ => "",
    };
    children.push(text_area_field(
        "other_instructions",
        "i18n:host.other.instructions",
        instructions,
    ));
}

// ── Layer + arrival + reveal ─────────────────────────────────────────────────

fn push_building_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
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
    let note = config
        .building_access
        .as_ref()
        .and_then(|b| b.note.as_deref())
        .unwrap_or("");
    children.push(text_field(
        "building_access_gate_code",
        "i18n:host.building.gateCode",
        gate,
    ));
    children.push(text_field(
        "building_access_intercom",
        "i18n:host.building.intercom",
        intercom,
    ));
    children.push(text_area_field(
        "building_access_note",
        "i18n:host.building.note",
        note,
    ));
}

fn push_parking_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    let info = config
        .parking
        .as_ref()
        .map(|p| p.info.as_str())
        .unwrap_or("");
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
    children.push(text_field("parking_info", "i18n:host.parking.info", info));
    children.push(text_field(
        "parking_map_url",
        "i18n:host.parking.mapUrl",
        map_url,
    ));
    children.push(text_field("parking_code", "i18n:host.parking.code", code));
}

fn push_arrival_fields(children: &mut Vec<Component>, config: &ModuleConfig) {
    children.push(text_field(
        "address",
        "i18n:host.address.label",
        &config.arrival.address,
    ));
    children.push(text_field(
        "arrival_video_url",
        "i18n:host.video.label",
        &config.arrival.arrival_video_url,
    ));
    children.push(text_area_field(
        "global_note",
        "i18n:host.note.label",
        &config.arrival.global_note,
    ));

    let steps = config.parse_steps();
    for index in 0..STEP_SLOTS {
        push_step_slot(children, index, steps.get(index));
    }
}

fn push_reveal_policy(children: &mut Vec<Component>, policy: RevealPolicy) {
    children.push(
        Field::new()
            .name(json!("reveal_policy"))
            .label(json!("i18n:host.reveal.label"))
            .child(
                Select::new()
                    .name(json!("reveal_policy"))
                    .options(json!([
                        {"value": "always", "label": "i18n:host.reveal.always"},
                        {"value": "hours_before_24", "label": "i18n:host.reveal.hoursBefore24"},
                        {"value": "day_before_16h", "label": "i18n:host.reveal.dayBefore16h"},
                        {"value": "at_checkin", "label": "i18n:host.reveal.atCheckin"}
                    ]))
                    .value(json!(reveal_policy_str(policy))),
            )
            .into(),
    );
}

// ── Field helpers ────────────────────────────────────────────────────────────

fn text_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(json!(name))
        .label(json!(label_key))
        .child(TextInput::new().name(json!(name)).value(json!(value)))
        .into()
}

fn text_area_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(json!(name))
        .label(json!(label_key))
        .child(TextArea::new().name(json!(name)).value(json!(value)))
        .into()
}

fn push_step_slot(children: &mut Vec<Component>, index: usize, step: Option<&AccessStep>) {
    let slot = index + 1;
    let kind = step.and_then(|s| s.kind.as_deref()).unwrap_or("");
    let title_fr = step.map(|s| s.title.fr.as_str()).unwrap_or("");
    let title_en = step.map(|s| s.title.en.as_str()).unwrap_or("");
    let detail_fr = step
        .and_then(|s| s.detail.as_ref())
        .map(|d| d.fr.as_str())
        .unwrap_or("");
    let detail_en = step
        .and_then(|s| s.detail.as_ref())
        .map(|d| d.en.as_str())
        .unwrap_or("");

    children.push(
        Text::new()
            .text(json!(format!("i18n:host.step.slot{slot}")))
            .variant(json!("caption"))
            .into(),
    );
    children.push(
        Field::new()
            .name(json!(format!("steps.{index}.kind")))
            .label(json!("i18n:host.step.kind"))
            .child(
                Select::new()
                    .name(json!(format!("steps.{index}.kind")))
                    .options(json!([
                        {"value": "", "label": "i18n:host.step.kind.other"},
                        {"value": "parking", "label": "i18n:host.step.kind.parking"},
                        {"value": "door", "label": "i18n:host.step.kind.door"},
                        {"value": "elevator", "label": "i18n:host.step.kind.elevator"}
                    ]))
                    .value(json!(kind)),
            )
            .into(),
    );
    children.push(text_field(
        &format!("steps.{index}.title_fr"),
        "i18n:host.step.titleFr",
        title_fr,
    ));
    children.push(text_field(
        &format!("steps.{index}.title_en"),
        "i18n:host.step.titleEn",
        title_en,
    ));
    children.push(text_field(
        &format!("steps.{index}.detail_fr"),
        "i18n:host.step.detailFr",
        detail_fr,
    ));
    children.push(text_field(
        &format!("steps.{index}.detail_en"),
        "i18n:host.step.detailEn",
        detail_en,
    ));
}
