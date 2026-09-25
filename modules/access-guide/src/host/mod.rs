//! Host dashboard surfaces — conditional access configuration form (Wasm SDUI).

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    AddressMapPicker, Card, ChoiceList, Field, FieldHint, Form, Grid, InlineNotice, Page,
    RichTextEditor, SecretInput, Select, Stack, StepList, Text, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{
    coord_pair, host_lang, HostConfig, PrimaryMethod, RevealPolicy, StepRow, StepTextRow,
};

const STEP_SLOTS: usize = 8;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyWorkspaceTab,
    design_id = DesignId::AccessEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Key
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = HostConfig::read(&ctx)?;
    let texts = Texts::of(&config, host_lang(&ctx.locale));
    let saved_method = config.method();
    let draft_method = ctx
        .input_str("primary_method")
        .and_then(parse_primary_method)
        .or(saved_method);
    let method = draft_method.unwrap_or_default();
    let building_enabled =
        ctx.input_bool("building_access_enabled", config.building_access_enabled);
    let parking_enabled = ctx.input_bool("parking_enabled", config.parking_enabled);
    let steps_count = draft_steps_count(&ctx, &config);

    // Reveal timing only applies when there is (or can be) a code: primary
    // method with credential, and/or optional building / parking layers.
    let show_reveal = method.involves_access_code() || building_enabled || parking_enabled;

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
            .children(method_detail_children(method, &config, &texts))
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
                .children(vec![reveal_choice_list(config.reveal()).into()])
                .into(),
        );
    }

    form_children.push(
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
    );

    // No Save button — the modules drawer saves the form (`updateConfig`, taken by the platform).
    Ok(
        Surface::new(Page::new().child(Form::new().children(form_children)))
            .with_id(crate::ids::HOST_MAIN),
    )
}

/// The copy the form edits, in the host's language: its fields are named `<key>_<lang>`.
struct Texts<'a> {
    lang: &'static str,
    method_instructions: &'a str,
    building_note: &'a str,
    parking_info: &'a str,
    global_note: &'a str,
    steps: &'a [StepTextRow],
}

impl<'a> Texts<'a> {
    fn of(config: &'a HostConfig, lang: &'static str) -> Self {
        if lang == "en" {
            Self {
                lang,
                method_instructions: &config.method_instructions_en,
                building_note: &config.building_note_en,
                parking_info: &config.parking_info_en,
                global_note: &config.global_note_en,
                steps: &config.steps_en,
            }
        } else {
            Self {
                lang,
                method_instructions: &config.method_instructions_fr,
                building_note: &config.building_note_fr,
                parking_info: &config.parking_info_fr,
                global_note: &config.global_note_fr,
                steps: &config.steps_fr,
            }
        }
    }

    fn name(&self, key: &str) -> String {
        format!("{key}_{}", self.lang)
    }
}

// ── Draft helpers ────────────────────────────────────────────────────────────

fn draft_steps_count(ctx: &HostContext, config: &HostConfig) -> usize {
    if let Some(n) = ctx.input_u64("steps_count") {
        return (n as usize).min(STEP_SLOTS);
    }
    config.live_steps().count().min(STEP_SLOTS)
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

// ── Choice lists ─────────────────────────────────────────────────────────────

/// No method preselected until the host picks one: the platform blocks publication meanwhile.
fn method_choice_list(selected: Option<PrimaryMethod>) -> ChoiceList {
    let list = ChoiceList::new();
    let list = match selected {
        Some(method) => list.value(method.as_wire()),
        None => list,
    };
    list.name("primary_method")
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
    config: &HostConfig,
    texts: &Texts,
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

/// One field per language for the instructions of whichever method is chosen.
fn instructions_field(texts: &Texts, label_key: &str) -> Component {
    rich_text_field(
        &texts.name("method_instructions"),
        label_key,
        texts.method_instructions,
    )
}

fn push_keybox_fields(children: &mut Vec<Component>, config: &HostConfig, texts: &Texts) {
    children.push(text_field(
        "keybox_location",
        "i18n:host.keybox.location",
        &config.keybox_location,
    ));
    children.push(
        FieldHint::new()
            .text("i18n:host.keybox.location.hint")
            .into(),
    );
    children.push(secret_field(
        "keybox_code",
        "i18n:host.keybox.code",
        &config.keybox_code,
    ));
    children.push(FieldHint::new().text("i18n:host.keybox.code.hint").into());
    children.push(instructions_field(texts, "i18n:host.keybox.instructions"));
}

fn push_door_code_fields(children: &mut Vec<Component>, config: &HostConfig, texts: &Texts) {
    let target = match config.door_code_target.trim() {
        "" => "building",
        target => target,
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
                    .value(target),
            )
            .into(),
    );
    children.push(
        FieldHint::new()
            .text("i18n:host.doorCode.target.hint")
            .into(),
    );
    children.push(secret_field(
        "door_code",
        "i18n:host.doorCode.code",
        &config.door_code,
    ));
    children.push(FieldHint::new().text("i18n:host.doorCode.code.hint").into());
    children.push(instructions_field(texts, "i18n:host.doorCode.instructions"));
}

fn push_smart_lock_fields(children: &mut Vec<Component>, config: &HostConfig, texts: &Texts) {
    children.push(secret_field(
        "smart_lock_manual_code",
        "i18n:host.smartLock.manualCode",
        &config.smart_lock_manual_code,
    ));
    children.push(
        FieldHint::new()
            .text("i18n:host.smartLock.manualCode.hint")
            .into(),
    );
    children.push(instructions_field(
        texts,
        "i18n:host.smartLock.instructions",
    ));
}

fn push_smart_lock_binding(children: &mut Vec<Component>, config: &HostConfig) {
    let provider = config.smart_lock_provider_module_id.trim();
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

fn push_in_person_fields(children: &mut Vec<Component>, config: &HostConfig) {
    let mut picker = AddressMapPicker::new()
        .label("i18n:host.inPerson.meetingPlace")
        .hint("i18n:host.inPerson.meetingPlace.hint")
        .addressName("in_person_meeting_place")
        .latName("in_person_meeting_lat")
        .lngName("in_person_meeting_lng")
        .address(config.in_person_meeting_place.as_str());
    // No point placed: no position at all, never 0, 0.
    if let (Some(lat), Some(lng)) =
        coord_pair(&config.in_person_meeting_lat, &config.in_person_meeting_lng)
    {
        picker = picker.lat(lat).lng(lng);
    }
    children.push(picker.into());
    children.push(text_field(
        "in_person_time_hint",
        "i18n:host.inPerson.timeHint",
        &config.in_person_time_hint,
    ));
    children.push(text_field(
        "in_person_contact",
        "i18n:host.inPerson.contact",
        &config.in_person_contact,
    ));
}

fn push_building_staff_fields(children: &mut Vec<Component>, config: &HostConfig) {
    let kind = match config.building_staff_kind.trim() {
        "" => "reception",
        kind => kind,
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
                    .value(kind),
            )
            .into(),
    );
    children.push(text_field(
        "building_staff_desk_location",
        "i18n:host.buildingStaff.deskLocation",
        &config.building_staff_desk_location,
    ));
    children.push(text_field(
        "building_staff_hours",
        "i18n:host.buildingStaff.hours",
        &config.building_staff_hours,
    ));
    children.push(text_field(
        "building_staff_contact",
        "i18n:host.buildingStaff.contact",
        &config.building_staff_contact,
    ));
}

fn push_host_greets_fields(children: &mut Vec<Component>, config: &HostConfig) {
    children.push(rich_text_field(
        "host_greets_contact_note",
        "i18n:host.hostGreets.contactNote",
        &config.host_greets_contact_note,
    ));
    children.push(text_field(
        "host_greets_eta_hint",
        "i18n:host.hostGreets.etaHint",
        &config.host_greets_eta_hint,
    ));
}

fn push_other_fields(children: &mut Vec<Component>, texts: &Texts) {
    children.push(instructions_field(texts, "i18n:host.other.instructions"));
}

// ── Layers ───────────────────────────────────────────────────────────────────

fn layer_card_building(enabled: bool, config: &HostConfig, texts: &Texts) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("building_access_enabled")
        .label("i18n:host.building.enabled")
        .checked(enabled)
        .into()];
    if enabled {
        children.push(secret_field(
            "building_access_gate_code",
            "i18n:host.building.gateCode",
            &config.building_access_gate_code,
        ));
        children.push(text_field(
            "building_access_intercom",
            "i18n:host.building.intercom",
            &config.building_access_intercom,
        ));
        children.push(rich_text_field(
            &texts.name("building_note"),
            "i18n:host.building.note",
            texts.building_note,
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

fn layer_card_parking(enabled: bool, config: &HostConfig, texts: &Texts) -> Component {
    let mut children: Vec<Component> = vec![ToggleRow::new()
        .name("parking_enabled")
        .label("i18n:host.parking.enabled")
        .checked(enabled)
        .into()];
    if enabled {
        children.push(rich_text_field(
            &texts.name("parking_info"),
            "i18n:host.parking.info",
            texts.parking_info,
        ));
        children.push(text_field(
            "parking_map_url",
            "i18n:host.parking.mapUrl",
            &config.parking_map_url,
        ));
        children.push(secret_field(
            "parking_code",
            "i18n:host.parking.code",
            &config.parking_code,
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
        .title("i18n:host.section.parking")
        .subtitle("i18n:host.section.parking.help")
        .icon(IconName::Car)
        .children(children)
        .into()
}

// ── Arrival ──────────────────────────────────────────────────────────────────

fn arrival_children(config: &HostConfig, texts: &Texts, steps_count: usize) -> Vec<Component> {
    let mut children: Vec<Component> = Vec::new();
    children.push(
        AddressMapPicker::new()
            .label("i18n:host.address.label")
            .hint("i18n:host.address.hint")
            .addressName("address")
            .latName("arrival_lat")
            .lngName("arrival_lng")
            .address(config.address.as_str())
            .into(),
    );

    // Rows are renumbered on each render: the copy follows its step by index.
    let steps: Vec<(&StepRow, Option<&StepTextRow>)> = config
        .live_steps()
        .map(|(index, row)| (row, texts.steps.get(index)))
        .collect();
    let step_rows: Vec<Component> = (0..steps_count)
        .map(|index| {
            let (row, text) = steps.get(index).copied().unzip();
            step_row(index, texts, row, text.flatten())
        })
        .collect();

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
        &config.arrival_video_url,
    ));
    children.push(rich_text_field(
        &texts.name("global_note"),
        "i18n:host.note.label",
        texts.global_note,
    ));
    children
}

/// `steps.N.kind` is the shared skeleton; `steps_<lang>.N.title` / `.detail` its copy. The step
/// list only blanks `steps.N.*` on removal: a blank kind is what marks the row removed.
fn step_row(
    index: usize,
    texts: &Texts,
    step: Option<&StepRow>,
    text: Option<&StepTextRow>,
) -> Component {
    let kind = step
        .and_then(|s| s.kind.as_deref())
        .filter(|k| !k.trim().is_empty())
        .unwrap_or("other");
    let title = text.map(|t| t.title.as_str()).unwrap_or("");
    let detail = text.map(|t| t.detail.as_str()).unwrap_or("");
    let copy = texts.name("steps");

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
                &format!("{copy}.{index}.title"),
                "i18n:host.step.title",
                title,
            ),
            text_field(
                &format!("{copy}.{index}.detail"),
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

/// A code is never sent back to the form: blank keeps it (the platform ignores `""`).
fn secret_field(name: &str, label_key: &str, saved: &str) -> Component {
    let mut input = SecretInput::new().name(name).value(String::new());
    if !saved.trim().is_empty() {
        input = input.placeholder("i18n:host.secret.keep");
    }
    Field::new().name(name).label(label_key).child(input).into()
}

fn rich_text_field(name: &str, label_key: &str, value: &str) -> Component {
    Field::new()
        .name(name)
        .label(label_key)
        .child(RichTextEditor::new().name(name).value(value))
        .into()
}
