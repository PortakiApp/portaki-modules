//! Module queries — appliance guide content.

use portaki_sdk::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{Appliance, AppliancesBundle, AppliancesPayload};
use crate::store;

#[portaki_sdk::params]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetContentArgs {
    pub locale: Option<String>,
    /// Optional device id for detail views (also used when overlay args are forwarded).
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliancesContentView {
    pub devices: Vec<Appliance>,
    /// Global TipTap JSON safety notice.
    #[serde(rename = "safetyNotice")]
    pub safety_notice: String,
    /// Canonical JSON for the resolved locale.
    pub content: String,
    /// Legacy slots — kept for host tooling during transition.
    pub content_fr: String,
    pub content_en: String,
}

#[portaki_sdk::query(
    name = "getContent",
    example(label = "Contenu en français", input = r#"{"locale":"fr-FR"}"#),
    example(label = "Contenu en anglais", input = r#"{"locale":"en-US"}"#)
)]
pub fn get_content(ctx: Context, args: GetContentArgs) -> Result<AppliancesContentView> {
    let locale = args.locale.unwrap_or_else(|| ctx.locale.clone());
    let row = store::load_content()?;
    let (content_fr, content_en) = match row {
        Some(row) => (row.content_fr, row.content_en),
        None => (String::new(), String::new()),
    };
    let payload =
        AppliancesBundle::from_row(&content_fr, &content_en).pick(&locale, &ctx.property.locale);
    let content = payload
        .to_json_string()
        .unwrap_or_else(|_| content_fr.clone());
    Ok(AppliancesContentView {
        devices: payload.devices,
        safety_notice: payload.safety_notice,
        content,
        content_fr,
        content_en,
    })
}

pub fn load_payload(ctx: &Context) -> Result<AppliancesPayload> {
    store::load_payload_for(&ctx.locale, &ctx.property.locale)
}

/// Ce qui bloque la publication : les longueurs de la spec Appareils et le nombre d'appareils
/// en avant ([`AppliancesPayload::problems`]), chacun désignant son champ (`config.<clé>`) ; et,
/// sans bloquer, le modèle, la vidéo et les étapes ([`AppliancesPayload::warnings`]).
#[portaki_sdk::query(name = "publishReadiness", example(label = "Prêt à publier ?"))]
pub fn publish_readiness(
    ctx: Context,
) -> Result<portaki_sdk::contracts::publish::PublishReadiness> {
    use portaki_sdk::contracts::publish::{PublishCheck, PublishLevel, PublishReadiness};
    let payload = store::load_payload_for(&ctx.locale, &ctx.property.locale)?;
    let required = payload
        .problems()
        .into_iter()
        .map(|problem| (problem, PublishLevel::Required));
    let recommended = payload
        .warnings()
        .into_iter()
        .map(|warning| (warning, PublishLevel::Recommended));
    let items = required
        .chain(recommended)
        .map(|((field, error), level)| PublishCheck {
            label: crate::i18n::text(match field.rsplit('.').next().unwrap_or_default() {
                "paperManualsLocation" => "host.paperManuals.label",
                "featuredLimit" => "host.featured.label",
                "manualUrl" => "host.device.manualUrl",
                "location" => "host.device.location",
                "description" => "host.device.description",
                "safetyNote" => "host.device.safetyNote",
                "model" => "host.device.model",
                "videoUrl" => "host.device.videoUrl",
                "steps" => "host.device.steps",
                "devices" => "host.list.title",
                _ => "host.device.name",
            }),
            id: format!("config.{field}"),
            level,
            ok: false,
            hint: error,
        })
        .collect();
    Ok(PublishReadiness { items })
}
