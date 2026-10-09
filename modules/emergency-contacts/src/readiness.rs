//! Publication readiness: the field errors of [`ModuleConfig::problems`], each naming its field,
//! and the warning of a contact that repeats the host's own number (spec Urgences §3).

use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
use portaki_sdk::prelude::*;

use crate::config::{compact, ModuleConfig};

#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(ctx: Context) -> Result<PublishReadiness> {
    let config = ModuleConfig::load(&ctx)?;
    let mut items: Vec<PublishCheck> = config
        .problems()
        .into_iter()
        .map(|(field, error)| PublishCheck {
            label: crate::i18n::text(field_label(&field)),
            id: format!("config.{field}"),
            level: PublishLevel::Required,
            ok: false,
            hint: error,
        })
        .collect();

    // Le numéro de l'hôte en double dans ses contacts : deux rangées qui sonnent au même endroit.
    let host = ctx
        .host
        .as_ref()
        .and_then(|host| host.phone.as_deref())
        .map(compact)
        .unwrap_or_default();
    let host = Some(compact(config.host_visible_phone.trim()))
        .filter(|phone| !phone.is_empty())
        .unwrap_or(host);
    if config.show_host() && !host.is_empty() {
        if let Some(index) = config
            .contacts
            .iter()
            .position(|contact| compact(contact.phone.trim()) == host)
        {
            items.push(PublishCheck {
                id: format!("config.contacts.{index}.phone"),
                level: PublishLevel::Recommended,
                ok: false,
                label: crate::i18n::text("host.contact.phone"),
                hint: crate::i18n::text("publish.duplicate.hint"),
            });
        }
    }
    // Une épingle loin du logement : sans doute posée au mauvais endroit (§2.3 : pharmacie à
    // 30 km au plus, hôpital à 50). Un avertissement, pas un refus.
    if let Some(home) = ctx.property.coordinates {
        for (key, lat, lng, km) in [
            ("pharmacy", config.pharmacy_lat, config.pharmacy_lng, 30.0),
            ("hospital", config.hospital_lat, config.hospital_lng, 50.0),
        ] {
            if let (Some(lat), Some(lng)) = (lat, lng) {
                if !portaki_sdk::config::check::within_km(lat, lng, home.lat, home.lng, km) {
                    items.push(PublishCheck {
                        id: format!("config.{key}_lat"),
                        level: PublishLevel::Recommended,
                        ok: false,
                        label: crate::i18n::text(&format!("host.{key}.position")),
                        hint: crate::i18n::text(&format!("publish.far.{key}")),
                    });
                }
            }
        }
    }
    Ok(PublishReadiness { items })
}

/// Le libellé du champ en défaut : celui du formulaire, sans l'index de la ligne.
fn field_label(field: &str) -> &'static str {
    match field.rsplit('.').next().unwrap_or_default() {
        "host_hours_from" => "host.hours.from",
        "host_hours_to" => "host.hours.to",
        "label" => "host.contact.label",
        "phone" => "host.contact.phone",
        "note" => "host.contact.note",
        "host_visible_phone" => "host.phone.label",
        "pharmacy_phone" => "host.pharmacyPhone.label",
        "hospital_phone" => "host.hospitalPhone.label",
        "doctor_phone" => "host.doctorPhone.label",
        _ => "host.contacts.title",
    }
}

#[portaki_sdk::wire]
#[derive(PartialEq)]
pub struct MapMarkersResponse {
    pub markers: Vec<MapMarker>,
}

/// La pharmacie et l'hôpital sur la Carte du livret (spec Urgences §2.3). Sans position, pas de
/// repère : un repère au hasard vaut moins qu'un repère absent.
#[portaki_sdk::query(name = "mapMarkers", example(label = "Pharmacie et hôpital"))]
pub fn map_markers(ctx: Context) -> Result<MapMarkersResponse> {
    let config = ModuleConfig::load(&ctx)?;
    let markers = [
        (
            "pharmacy",
            &config.pharmacy,
            config.pharmacy_lat,
            config.pharmacy_lng,
        ),
        (
            "hospital",
            &config.hospital,
            config.hospital_lat,
            config.hospital_lng,
        ),
    ]
    .into_iter()
    .filter_map(|(id, name, lat, lng)| {
        let mut marker = MapMarker::new(id, lat?, lng?).kind(MapMarkerKind::Poi);
        let name = name.trim();
        marker = marker.label(if name.is_empty() {
            format!("i18n:guest.health.{id}")
        } else {
            name.to_string()
        });
        // La catégorie range le repère dans le pratique et lui donne son pictogramme santé.
        marker.category = Some(id.into());
        Some(marker)
    })
    .collect();
    Ok(MapMarkersResponse { markers })
}
