//! Host dashboard surfaces — conditional access configuration form (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Button, Card, ChoiceList, Field, FieldHint, Form, Grid, InlineNotice, Page,
    RichTextEditor, SecretInput, Select, Stack, StepList, Text, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::{json, Value};

use crate::config::{
    load_config, AccessStep, DoorCodeTarget, MethodFields, ModuleConfig, PrimaryMethod,
    RevealPolicy, StaffKind,
};
use crate::texts::{load_texts_for_host, ModuleTexts};

const STEP_SLOTS: usize = 8;
const EMIT_SURFACE_INPUT: &str = "host.surface.input";

#[portaki_sdk::surface(host, id = "main")]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let config = load_config().unwrap_or_default();
    let texts = load_texts_for_host(&ctx.locale).unwrap_or_default();
    let draft_method = draft_primary_method(&ctx.input, &config);
    let building_enabled = draft_flag(
        &ctx.input,
        "building_access_enabled",
        config.building_access.is_some() || texts.building_note.is_some(),
    );
    let parking_enabled = draft_flag(
        &ctx.input,
        "parking_enabled",
        config.parking.is_some() || !texts.parking_info.trim().is_empty(),
    );
    let steps_count = draft_steps_count(&ctx.input, &config);

    let submit_args = json!({
        "primary_method": draft_method.as_wire(),
        "building_access_enabled": building_enabled,
        "parking_enabled": parking_enabled,
        "reveal_policy": config.reveal_policy.as_wire(),
    });
    let save_action =
        serde_json::to_value(Action::command("access-guide", "updateConfig", submit_args))
            .unwrap_or(json!({}));

    let form_children: Vec<Component> = vec![
        Card::new()
            .title(json!("i18n:host.section.primary"))
            .subtitle(json!("i18n:host.section.primary.help"))
            .icon(json!("key"))
            .children(vec![method_choice_list(draft_method).into()])
            .into(),
        Card::new()
            .title(json!("i18n:host.section.methodDetails"))
            .subtitle(json!("i18n:host.section.methodDetails.help"))
            .icon(json!("lock"))
            .children(method_detail_children(draft_method, &config, &texts))
            .into(),
        Grid::new()
            .columns(json!(2))
            .gap(json!(16))
            .children(vec![
                layer_card_building(building_enabled, &config, &texts),
                layer_card_parking(parking_enabled, &config, &texts),
            ])
            .into(),
        Card::new()
            .title(json!("i18n:host.section.arrival"))
            .subtitle(json!("i18n:host.section.arrival.help"))
            .icon(json!("map-pin"))
            .children(arrival_children(&config, &texts, steps_count))
            .into(),
        Card::new()
            .title(json!("i18n:host.section.reveal"))
            .subtitle(json!("i18n:host.section.reveal.help"))
            .icon(json!("clock-circle"))
            .children(vec![reveal_choice_list(config.reveal_policy).into()])
            .into(),
        Text::new()
            .text(json!("i18n:host.main.help"))
            .variant(json!("caption"))
            .into(),
        Button::new()
            .label(json!("i18n:host.save"))
            .action(save_action)
            .tone(Tone::Primary)
            .into(),
    ];

    Surface::new(Page::new().child(Form::new().children(form_children))).with_id("main")
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

fn draft_steps_count(input: &Value, config: &ModuleConfig) -> usize {
    if let Some(n) = input.get("steps_count").and_then(|v| v.as_u64()) {
        return (n as usize).min(STEP_SLOTS);
    }
    let existing = config.parse_steps().len();
    existing.min(STEP_SLOTS)
}

fn emit_input(payload: Value) -> Value {
    serde_json::to_value(Action::Emit {
        event: EMIT_SURFACE_INPUT.into(),
        payload: Some(payload),
    })
    .unwrap_or(json!({}))
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
        .name(json!("primary_method"))
        .value(json!(selected.as_wire()))
        .emitOnChange(json!(true))
        .layout(json!("cards"))
        .choices(json!([
            {
                "value": PrimaryMethod::Keybox.as_wire(),
                "label": "i18n:host.method.keybox",
                "description": "i18n:host.method.keybox.desc",
                "icon": "key"
            },
            {
                "value": PrimaryMethod::DoorCode.as_wire(),
                "label": "i18n:host.method.door_code",
                "description": "i18n:host.method.door_code.desc",
                "icon": "grid"
            },
            {
                "value": PrimaryMethod::SmartLock.as_wire(),
                "label": "i18n:host.method.smart_lock",
                "description": "i18n:host.method.smart_lock.desc",
                "icon": "lock"
            },
            {
                "value": PrimaryMethod::InPerson.as_wire(),
                "label": "i18n:host.method.in_person",
                "description": "i18n:host.method.in_person.desc",
                "icon": "users"
            },
            {
                "value": PrimaryMethod::BuildingStaff.as_wire(),
                "label": "i18n:host.method.building_staff",
                "description": "i18n:host.method.building_staff.desc",
                "icon": "building"
            },
            {
                "value": PrimaryMethod::HostGreets.as_wire(),
                "label": "i18n:host.method.host_greets",
                "description": "i18n:host.method.host_greets.desc",
                "icon": "smile"
            },
            {
                "value": PrimaryMethod::Other.as_wire(),
                "label": "i18n:host.method.other",
                "description": "i18n:host.method.other.desc",
                "icon": "more-horizontal"
            }
        ]))
}

fn reveal_choice_list(policy: RevealPolicy) -> ChoiceList {
    ChoiceList::new()
        .name(json!("reveal_policy"))
        .value(json!(policy.as_wire()))
        .layout(json!("compact"))
        .choices(json!([
            {
                "value": RevealPolicy::Always.as_wire(),
                "label": "i18n:host.reveal.always",
                "description": "i18n:host.reveal.always.desc",
                "icon": "clock-circle"
            },
            {
                "value": RevealPolicy::HoursBefore24.as_wire(),
                "label": "i18n:host.reveal.hoursBefore24",
                "description": "i18n:host.reveal.hoursBefore24.desc",
                "icon": "clock-circle"
            },
            {
                "value": RevealPolicy::DayBefore16h.as_wire(),
                "label": "i18n:host.reveal.dayBefore16h",
                "description": "i18n:host.reveal.dayBefore16h.desc",
                "icon": "clock-circle"
            },
            {
                "value": RevealPolicy::AtCheckin.as_wire(),
                "label": "i18n:host.reveal.atCheckin",
                "description": "i18n:host.reveal.atCheckin.desc",
                "icon": "clock-circle"
            }
        ]))
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

fn method_instructions<'a>(texts: &'a ModuleTexts) -> &'a str {
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
            .text(json!("i18n:host.keybox.location.hint"))
            .into(),
    );
    children.push(secret_field("keybox_code", "i18n:host.keybox.code", code));
    children.push(
        FieldHint::new()
            .text(json!("i18n:host.keybox.code.hint"))
            .into(),
    );
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
    children.push(
        FieldHint::new()
            .text(json!("i18n:host.doorCode.target.hint"))
            .into(),
    );
    children.push(secret_field("door_code", "i18n:host.doorCode.code", code));
    children.push(
        FieldHint::new()
            .text(json!("i18n:host.doorCode.code.hint"))
            .into(),
    );
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
            .text(json!("i18n:host.smartLock.manualCode.hint"))
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
                        let mut seen = std::collections::BTreeSet::new();
                        for peer in &peers {
                            if peer.module_id.trim().is_empty()
                                || !seen.insert(peer.module_id.clone())
                            {
                                continue;
                            }
                            let label = if peer.display_name.trim().is_empty() {
                                peer.module_id.clone()
                            } else {
                                peer.display_name.clone()
                            };
                            opts.push(json!({
                                "value": peer.module_id,
                                "label": label
                            }));
                        }
                        // Keep a previously saved id selectable even if not installed yet.
                        if !provider.is_empty() && !seen.contains(provider) {
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
    let notice = if provider.is_empty() {
        "i18n:host.smartLock.provider.notice.manual"
    } else {
        "i18n:host.smartLock.provider.notice.linked"
    };
    let mut banner = InlineNotice::new().message(json!(notice));
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
            .label(json!("i18n:host.inPerson.meetingPlace"))
            .hint(json!("i18n:host.inPerson.meetingPlace.hint"))
            .addressName(json!("in_person_meeting_place"))
            .latName(json!("in_person_meeting_lat"))
            .lngName(json!("in_person_meeting_lng"))
            .address(json!(place))
            .lat(json!(format_coord(lat)))
            .lng(json!(format_coord(lng)))
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
        .name(json!("building_access_enabled"))
        .label(json!("i18n:host.building.enabled"))
        .checked(json!(enabled))
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
                .text(json!("i18n:host.layer.disabled"))
                .variant(json!("caption"))
                .into(),
        );
    }
    Card::new()
        .title(json!("i18n:host.section.building"))
        .subtitle(json!("i18n:host.section.building.help"))
        .icon(json!("building"))
        .children(children)
        .into()
}

fn layer_card_parking(enabled: bool, config: &ModuleConfig, texts: &ModuleTexts) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name(json!("parking_enabled"))
        .label(json!("i18n:host.parking.enabled"))
        .checked(json!(enabled))
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
                .text(json!("i18n:host.layer.disabled"))
                .variant(json!("caption"))
                .into(),
        );
    }
    Card::new()
        .title(json!("i18n:host.section.parking"))
        .subtitle(json!("i18n:host.section.parking.help"))
        .icon(json!("car"))
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
            .label(json!("i18n:host.address.label"))
            .hint(json!("i18n:host.address.hint"))
            .addressName(json!("address"))
            .latName(json!("arrival_lat"))
            .lngName(json!("arrival_lng"))
            .address(json!(config.arrival.address.as_str()))
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
            .label(json!("i18n:host.steps.label"))
            .hint(json!("i18n:host.steps.hint"))
            .emptyTitle(json!("i18n:host.steps.emptyTitle"))
            .emptyDescription(json!("i18n:host.steps.emptyDescription"))
            .addLabel(json!("i18n:host.steps.add"))
            .removeLabel(json!("i18n:host.steps.remove"))
            .itemKeyPrefix(json!("steps"))
            .addAction(emit_input(json!({
                "steps_count": (steps_count + 1).min(STEP_SLOTS)
            })))
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
        .gap(json!(10))
        .children(vec![
            Field::new()
                .name(json!(format!("steps.{index}.kind")))
                .label(json!("i18n:host.step.kind"))
                .child(
                    Select::new()
                        .name(json!(format!("steps.{index}.kind")))
                        .options(json!([
                            {"value": "parking", "label": "i18n:host.step.kind.parking"},
                            {"value": "door", "label": "i18n:host.step.kind.door"},
                            {"value": "elevator", "label": "i18n:host.step.kind.elevator"},
                            {"value": "other", "label": "i18n:host.step.kind.other"}
                        ]))
                        .value(json!(kind)),
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
        .name(json!(name))
        .label(json!(label_key))
        .child(TextInput::new().name(json!(name)).value(json!(value)))
        .into()
}

fn secret_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(json!(name))
        .label(json!(label_key))
        .child(SecretInput::new().name(json!(name)).value(json!(value)))
        .into()
}

fn rich_text_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(json!(name))
        .label(json!(label_key))
        .child(RichTextEditor::new().name(json!(name)).value(json!(value)))
        .into()
}
