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
