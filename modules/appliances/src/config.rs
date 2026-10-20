//! Les règles de publication qu'aucun champ ne dit.

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::resource::{Collection, Problem};

use crate::domain::is_video_url;
use crate::{Device, Display};

/// Plus d'appareils en avant que la carte n'en montre, ou une vidéo qu'aucun lecteur ne lit.
pub fn problems(devices: &Collection<Device, Display>) -> Vec<Problem> {
    let mut out = Vec::new();
    let featured = devices
        .rows()
        .iter()
        .filter(|d| d.featured && !d.hidden)
        .count();
    let limit = devices.meta().limit();
    if featured > limit {
        out.push(Problem {
            row: None,
            field: "featured_limit",
            message: too_many_featured(limit),
        });
    }
    for device in devices.rows() {
        if !is_video_url(device.video_url.trim()) {
            out.push(Problem {
                row: Some(device.id.clone()),
                field: "video_url",
                message: portaki_sdk::bundle_text!("host.video.invalid"),
            });
        }
    }
    out
}

/// « Vous avez déjà mis {count} appareils en avant. »
pub fn too_many_featured(limit: usize) -> I18nText {
    let count = limit.to_string();
    portaki_sdk::bundle_text!("host.featured.tooMany", &[("count", count.as_str())])
}
