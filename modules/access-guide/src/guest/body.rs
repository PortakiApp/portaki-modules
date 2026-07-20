//! Shared guest SDUI body for access guide.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::primitives::{
    Badge, Button, InfoBanner, KeyValue, Link, ListItem, Map, Text,
};
use serde_json::json;

use crate::config::{
    AccessStep, BuildingAccess, DoorCodeTarget, MethodFields, ParkingLayer, StaffKind,
};
use crate::reveal::SECRET_MASK;

use super::load::GuestData;

fn kind_label(kind: Option<&str>, locale: &str) -> String {
    let is_en = locale.to_ascii_lowercase().starts_with("en");
    match kind.map(str::trim).unwrap_or("") {
        "parking" => "Parking".to_string(),
        "door" => {
            if is_en {
                "Door".into()
            } else {
                "Porte".into()
            }
        }
        "elevator" => {
            if is_en {
                "Lift".into()
            } else {
                "Ascenseur".into()
            }
        }
        _ => {
            if is_en {
                "Step".into()
            } else {
                "Étape".into()
            }
        }
    }
}

fn external_action(url: &str) -> serde_json::Value {
    serde_json::to_value(Action::External {
        url: url.to_string(),
    })
    .unwrap_or(json!({}))
}

fn command_action(module_id: &str, name: &str, args: serde_json::Value) -> serde_json::Value {
    serde_json::to_value(Action::command(module_id, name, args)).unwrap_or(json!({}))
}

fn google_maps_search_url(lat: f64, lng: f64) -> String {
    format!("https://www.google.com/maps/search/?api=1&query={lat},{lng}")
}

/// Prefer parking plan URL, then in-person meeting GPS, then property GPS.
fn maps_url(data: &GuestData) -> Option<String> {
    if let Some(parking) = data.config.parking.as_ref() {
        let configured = parking.map_url.trim();
        if !configured.is_empty() {
            return Some(configured.to_string());
        }
    }
    if let Some((lat, lng)) = meeting_coords(data) {
        return Some(google_maps_search_url(lat, lng));
    }
    if data.lat != 0.0 || data.lng != 0.0 {
        return Some(google_maps_search_url(data.lat, data.lng));
    }
    None
}

fn meeting_coords(data: &GuestData) -> Option<(f64, f64)> {
    match &data.config.method {
        MethodFields::InPerson {
            lat: Some(lat),
            lng: Some(lng),
            ..
        } if *lat != 0.0 || *lng != 0.0 => Some((*lat, *lng)),
        _ => None,
    }
}

fn map_at(lat: f64, lng: f64) -> Component {
    Component::Map(
        Map::new()
            .viewport(json!({
                "center": { "lat": lat, "lng": lng },
                "zoom": 15
            }))
            .markers(json!([{
                "lat": lat,
                "lng": lng,
                "tone": "primary"
            }]))
            .isStatic(json!(true))
            .interactionMode(json!("none")),
    )
}

fn property_map(data: &GuestData) -> Option<Component> {
    if let Some((lat, lng)) = meeting_coords(data) {
        return Some(map_at(lat, lng));
    }
    if data.lat == 0.0 && data.lng == 0.0 {
        return None;
    }
    Some(map_at(data.lat, data.lng))
}

fn kv_row(key_i18n: &str, value: &str, mono: bool) -> Component {
    let mut row = KeyValue::new().key(json!(key_i18n)).value(json!(value));
    if mono {
        row = row.mono(json!(true));
    }
    Component::KeyValue(row)
}

fn secret_display(data: &GuestData, plaintext: &str) -> String {
    if data.secrets_revealed {
        plaintext.to_string()
    } else {
        SECRET_MASK.to_string()
    }
}

fn push_secret_row(children: &mut Vec<Component>, data: &GuestData, key_i18n: &str, code: &str) {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return;
    }
    children.push(kv_row(key_i18n, &secret_display(data, trimmed), true));
}

fn push_text_row(children: &mut Vec<Component>, key_i18n: &str, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    children.push(kv_row(key_i18n, trimmed, false));
}

fn has_any_secret(data: &GuestData) -> bool {
    method_has_secret(&data.config.method)
        || data
            .config
            .building_access
            .as_ref()
            .and_then(|b| b.gate_code.as_deref())
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false)
        || data
            .config
            .parking
            .as_ref()
            .and_then(|p| p.code.as_deref())
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false)
}

fn method_has_secret(method: &MethodFields) -> bool {
    match method {
        MethodFields::Keybox { code: Some(c), .. } => !c.trim().is_empty(),
        MethodFields::DoorCode { code, .. } => !code.trim().is_empty(),
        MethodFields::SmartLock {
            manual_code: Some(c),
            ..
        } => !c.trim().is_empty(),
        _ => false,
    }
}

fn door_target_key(target: DoorCodeTarget) -> &'static str {
    match target {
        DoorCodeTarget::Gate => "i18n:guest.doorCode.gate",
        DoorCodeTarget::Building => "i18n:guest.doorCode.building",
        DoorCodeTarget::Apartment => "i18n:guest.doorCode.apartment",
    }
}

fn staff_kind_key(kind: StaffKind) -> &'static str {
    match kind {
        StaffKind::Reception => "i18n:guest.buildingStaff.reception",
        StaffKind::Caretaker => "i18n:guest.buildingStaff.caretaker",
    }
}

fn smart_lock_command_args(data: &GuestData) -> serde_json::Value {
    let mut args = json!({});
    if let Some(stay_id) = data.stay_id {
        args["stayId"] = json!(stay_id.to_string());
    }
    args
}

fn push_smart_lock_ctas(children: &mut Vec<Component>, data: &GuestData) {
    let Some(provider) = data
        .config
        .smart_lock_provider_module_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    if !data.secrets_revealed {
        return;
    }

    let args = smart_lock_command_args(data);
    children.push(Component::Button(
        Button::new()
            .label(json!("i18n:guest.smartLock.unlock"))
            .action(command_action(provider, "unlock", args.clone())),
    ));
    children.push(Component::Button(
        Button::new()
            .label(json!("i18n:guest.smartLock.getCredential"))
            .variant(json!("outline"))
            .action(command_action(provider, "getGuestCredential", args)),
    ));
}

fn push_primary_method(children: &mut Vec<Component>, data: &GuestData, detailed: bool) {
    match &data.config.method {
        MethodFields::Keybox {
            location,
            code,
            instructions,
        } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.keybox",
                false,
            ));
            push_text_row(children, "i18n:guest.keybox.location", location);
            if let Some(code) = code {
                push_secret_row(children, data, "i18n:guest.keybox.code", code);
            }
            if detailed {
                if let Some(instructions) = instructions {
                    push_text_row(children, "i18n:guest.instructions", instructions);
                }
            }
        }
        MethodFields::DoorCode {
            target,
            code,
            instructions,
        } => {
            children.push(kv_row("i18n:guest.method", door_target_key(*target), false));
            push_secret_row(children, data, "i18n:guest.doorCode.code", code);
            if detailed {
                if let Some(instructions) = instructions {
                    push_text_row(children, "i18n:guest.instructions", instructions);
                }
            }
        }
        MethodFields::SmartLock {
            instructions,
            manual_code,
        } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.smartLock",
                false,
            ));
            let has_provider = data
                .config
                .smart_lock_provider_module_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_some();
            if has_provider {
                push_smart_lock_ctas(children, data);
            }
            if let Some(manual_code) = manual_code {
                push_secret_row(children, data, "i18n:guest.smartLock.manualCode", manual_code);
            }
            if detailed || !has_provider {
                if let Some(instructions) = instructions {
                    push_text_row(children, "i18n:guest.instructions", instructions);
                }
            }
        }
        MethodFields::InPerson {
            meeting_place,
            lat,
            lng,
            time_hint,
            contact,
        } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.inPerson",
                false,
            ));
            push_text_row(children, "i18n:guest.inPerson.meetingPlace", meeting_place);
            // Map + Open Maps use meeting GPS via property_map / maps_url when set.
            if let (Some(lat), Some(lng)) = (lat, lng) {
                if *lat != 0.0 || *lng != 0.0 {
                    children.push(kv_row(
                        "i18n:guest.inPerson.coords",
                        &format!("{lat:.5}, {lng:.5}"),
                        true,
                    ));
                }
            }
            if let Some(time_hint) = time_hint {
                push_text_row(children, "i18n:guest.inPerson.timeHint", time_hint);
            }
            if let Some(contact) = contact {
                push_text_row(children, "i18n:guest.inPerson.contact", contact);
            }
        }
        MethodFields::BuildingStaff {
            staff_kind,
            desk_location,
            hours,
            contact,
        } => {
            children.push(kv_row("i18n:guest.method", staff_kind_key(*staff_kind), false));
            push_text_row(
                children,
                "i18n:guest.buildingStaff.deskLocation",
                desk_location,
            );
            if let Some(hours) = hours {
                push_text_row(children, "i18n:guest.buildingStaff.hours", hours);
            }
            if let Some(contact) = contact {
                push_text_row(children, "i18n:guest.buildingStaff.contact", contact);
            }
        }
        MethodFields::HostGreets {
            contact_note,
            eta_hint,
        } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.hostGreets",
                false,
            ));
            if let Some(eta_hint) = eta_hint {
                push_text_row(children, "i18n:guest.hostGreets.etaHint", eta_hint);
            }
            if let Some(contact_note) = contact_note {
                push_text_row(children, "i18n:guest.hostGreets.contactNote", contact_note);
            }
        }
        MethodFields::Other { instructions } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.other",
                false,
            ));
            push_text_row(children, "i18n:guest.instructions", instructions);
        }
    }
}

fn push_building_access(
    children: &mut Vec<Component>,
    data: &GuestData,
    building: &BuildingAccess,
    detailed: bool,
) {
    if building.is_empty() {
        return;
    }
    if let Some(gate) = building.gate_code.as_deref() {
        push_secret_row(children, data, "i18n:guest.building.gateCode", gate);
    }
    if let Some(intercom) = building.intercom.as_deref() {
        push_text_row(children, "i18n:guest.building.intercom", intercom);
    }
    if detailed {
        if let Some(note) = building.note.as_deref() {
            push_text_row(children, "i18n:guest.building.note", note);
        }
    }
}

fn push_parking(
    children: &mut Vec<Component>,
    data: &GuestData,
    parking: &ParkingLayer,
    _detailed: bool,
) {
    if parking.is_empty() {
        return;
    }
    push_text_row(children, "i18n:guest.parking", &parking.info);
    if let Some(code) = parking.code.as_deref() {
        push_secret_row(children, data, "i18n:guest.parking.code", code);
    }
}

fn push_reveal_banner(children: &mut Vec<Component>, data: &GuestData) {
    if data.secrets_revealed || !has_any_secret(data) {
        return;
    }
    let Some(message) = data.reveal_locked_message.as_ref() else {
        return;
    };
    children.push(Component::InfoBanner(
        InfoBanner::new()
            .title(json!("i18n:guest.reveal.lockedTitle"))
            .message(json!(message.clone())),
    ));
}

fn push_arrival_extras(children: &mut Vec<Component>, data: &GuestData, steps: &[AccessStep]) {
    let video = data.config.arrival.arrival_video_url.trim();
    if !video.is_empty() {
        children.push(Component::Link(
            Link::new()
                .label(json!("i18n:guest.watchVideo"))
                .href(json!(video.to_string()))
                .action(external_action(video)),
        ));
    }

    for step in steps {
        let title = step.title.pick(&data.locale);
        let mut item = ListItem::new()
            .title(json!(title))
            .child(Badge::new().label(json!(kind_label(step.kind.as_deref(), &data.locale))));
        if let Some(detail) = step.detail.as_ref() {
            let text = detail.pick(&data.locale);
            if !text.trim().is_empty() {
                item = item.child(Text::new().text(json!(text)).variant(json!("caption")));
            }
        }
        children.push(Component::ListItem(item));
    }
}

pub fn build_access_glance(data: &GuestData) -> Vec<Component> {
    let mut children = Vec::new();

    push_reveal_banner(&mut children, data);

    if let Some(map) = property_map(data) {
        children.push(map);
    }

    if !data.address.is_empty() {
        children.push(kv_row("i18n:guest.address", &data.address, false));
    }

    push_primary_method(&mut children, data, false);

    if let Some(building) = data.config.building_access.as_ref() {
        push_building_access(&mut children, data, building, false);
    }
    if let Some(parking) = data.config.parking.as_ref() {
        push_parking(&mut children, data, parking, false);
    }

    if let Some(url) = maps_url(data) {
        children.push(Component::Button(
            Button::new()
                .label(json!("i18n:guest.openMaps"))
                .variant(json!("outline"))
                .action(external_action(&url)),
        ));
    }

    children
}

pub fn build_access_detail(data: &GuestData) -> Vec<Component> {
    let mut children = Vec::new();

    let note = data.config.arrival.global_note.trim();
    if !note.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title(json!("i18n:guest.note.title"))
                .message(json!(note.to_string())),
        ));
    }

    push_reveal_banner(&mut children, data);

    if let Some(map) = property_map(data) {
        children.push(map);
    }

    if !data.address.is_empty() {
        children.push(kv_row("i18n:guest.address", &data.address, false));
    }

    push_primary_method(&mut children, data, true);

    if let Some(building) = data.config.building_access.as_ref() {
        push_building_access(&mut children, data, building, true);
    }
    if let Some(parking) = data.config.parking.as_ref() {
        push_parking(&mut children, data, parking, true);
    }

    if let Some(url) = maps_url(data) {
        children.push(Component::Button(
            Button::new()
                .label(json!("i18n:guest.openMaps"))
                .variant(json!("outline"))
                .action(external_action(&url)),
        ));
    }

    let steps = data.config.parse_steps();
    push_arrival_extras(&mut children, data, &steps);

    children
}
