//! Module queries — the publication check the declared config cannot express.

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::{MethodFields, ModuleConfig};
use crate::i18n::text;

/// A code-bearing access method needs its code — the platform already blocks on the method.
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let host = crate::config::HostConfig::load(&ctx)?;
    let mut items: Vec<PublishCheck> = host
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            label: text(field_label(&field)),
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            hint: error,
        })
        .collect();
    // Un « Contact » qui ressemble à un numéro mal saisi : un avertissement, le champ est libre.
    items.extend(
        host.warnings()
            .into_iter()
            .map(|(field, hint)| PublishCheck {
                label: text(field_label(&field)),
                id: format!("config.{field}"),
                level: PublishLevel::Recommended,
                ok: false,
                hint,
            }),
    );
    // Dès la réservation : le code se lit même si le séjour est annulé ensuite (§3).
    if host.reveal_policy.trim() == "always" {
        items.push(PublishCheck {
            id: "config.reveal_policy".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: text("config.revealPolicy"),
            hint: text("publish.reveal.always"),
        });
    }
    if host.parking_too_far(ctx.property.coordinates) {
        items.push(PublishCheck {
            id: "config.parking_position".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: text("host.parking.position"),
            hint: text("publish.parking.far"),
        });
    }

    let config = ModuleConfig::read(&ctx)?;
    // Une serrure sans code de secours, hors ligne : le voyageur n'a plus qu'à appeler (§9 cas 8).
    if let (MethodFields::SmartLock { manual_code: None }, true) =
        (&config.method, config.code_by_lock)
    {
        items.push(PublishCheck {
            id: "config.smart_lock_manual_code".into(),
            level: PublishLevel::Recommended,
            ok: false,
            label: text("host.smartLock.manualCode"),
            hint: text("publish.backupCode.missing"),
        });
    }
    if !matches!(
        config.method,
        MethodFields::Keybox { .. }
            | MethodFields::DoorCode { .. }
            | MethodFields::SmartLock { .. }
    ) {
        return Ok(PublishReadiness { items });
    }
    let ok = !config.entry_code_missing();
    // Généré par la serrure : ce qui manque, c'est la serrure, pas un code (§2.4).
    let hint = if config.code_by_lock {
        "publish.lock.missing"
    } else {
        "publish.entry-code.hint"
    };
    items.insert(
        0,
        PublishCheck {
            id: "entry-code".into(),
            level: PublishLevel::Required,
            ok,
            label: text("publish.entry-code.label"),
            hint: text(hint),
        },
    );
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de l'étape.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "method_instructions" => "config.methodInstructions",
        "method_other" => "host.methodOther",
        "reveal_hours" => "host.reveal.hours",
        "handover_slot" => "host.handover.slot",
        "desk_hours" => "host.desk.hours",
        "keybox_location" => "host.keybox.location",
        "global_note" => "config.globalNote",
        "late_arrival_note" => "config.lateArrivalNote",
        "parking_info" => "config.parkingInfo",
        "building_access_intercom" => "host.building.intercom",
        "building_note" => "host.building.note",
        "building_floor" => "host.building.floor",
        "parking_spot" => "host.parking.spot",
        "parking_price" => "host.parking.price",
        "arrival_video_url" => "host.video.label",
        "in_person_contact" => "host.inPerson.contact",
        "building_staff_contact" => "host.buildingStaff.contact",
        "handover_name" => "host.handover.name",
        "handover_phone" => "host.handover.phone",
        "desk_phone" => "host.desk.phone",
        "desk_after_hours" => "host.desk.afterHours",
        "title" => "host.step.title",
        "detail" => "host.step.detail",
        _ => "host.steps.label",
    }
}

#[portaki_sdk::wire]
#[derive(PartialEq)]
pub struct MapMarkersResponse {
    pub markers: Vec<MapMarker>,
}

/// Le repère du parking sur la Carte du livret (§2.8) : seulement s'il y a un parking et que
/// l'épingle est posée. La catégorie le range dans le pratique et lui donne son pictogramme.
#[portaki_sdk::query(name = "mapMarkers", example(label = "Le parking sur la carte"))]
pub fn map_markers(ctx: Context) -> Result<MapMarkersResponse> {
    let host = crate::config::HostConfig::load(&ctx)?;
    let markers = host
        .parking_point()
        .map(|(lat, lng)| {
            let mut marker = MapMarker::new("parking", lat, lng)
                .kind(MapMarkerKind::Poi)
                .label("i18n:guest.parking");
            marker.category = Some("parking".into());
            marker
        })
        .into_iter()
        .collect();
    Ok(MapMarkersResponse { markers })
}
