//! Ce que les appareils savent d'eux-mêmes : la vue du livret, le mode d'emploi, l'emoji proposé,
//! et les règles qu'aucun champ ne dit.

mod guide;
mod howto;
mod suggest;

use portaki_sdk::contracts::i18n::I18nText;
use portaki_sdk::resource::{Collection, Problem};

pub use guide::{Appliance, Guide};
pub use howto::{extract_howto_steps, is_video_url, HowToStep};

use crate::{Device, Display};

/// Les appareils de la carte d'accueil : 4 tant que l'hôte n'a rien choisi.
pub const DEFAULT_FEATURED: usize = 4;

impl Display {
    /// Combien d'appareils la carte d'accueil met en avant, borné à 2..=6.
    pub fn limit(&self) -> usize {
        match self.featured_limit {
            0 => DEFAULT_FEATURED,
            n => (n as usize).clamp(2, 6),
        }
    }
}

impl Device {
    /// L'emoji de l'appareil, ou celui que son nom propose (§4) ; vide → l'icône prise.
    pub fn shown_emoji(&self, name: &str) -> String {
        match self.emoji.trim() {
            "" => suggest::emoji(name).unwrap_or_default().to_string(),
            emoji => emoji.to_string(),
        }
    }
}

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
