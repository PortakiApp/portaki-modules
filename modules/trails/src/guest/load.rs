//! Ce qu'une surface voyageur lit une fois : la configuration, la langue, et où est le logement.

use portaki_sdk::prelude::*;

use crate::config::{ModuleConfig, TrailRow};

pub struct GuestData {
    pub trails: Vec<TrailRow>,
    pub commune_url: Option<String>,
    pub property: Option<(f64, f64)>,
    pub locale: String,
}

pub fn load_guest_data(ctx: &GuestContext) -> Result<GuestData> {
    let config = ModuleConfig::load(ctx)?;
    Ok(GuestData {
        trails: config.parse_trails(),
        commune_url: config.commune_link().map(str::to_string),
        property: ctx.property.coordinates.map(|point| (point.lat, point.lng)),
        locale: ctx.locale.clone(),
    })
}

impl GuestData {
    pub fn title(&self, trail: &TrailRow) -> String {
        trail.title.get(&self.locale).trim().to_string()
    }

    /// À quelle distance du logement part cet itinéraire — quand on sait les deux.
    pub fn start_metres(&self, trail: &TrailRow) -> Option<f64> {
        Some(crate::format::haversine_metres(
            self.property?,
            trail.coordinates()?,
        ))
    }

    /// Le départ est-il assez loin pour mériter un itinéraire ?
    ///
    /// Sans position — du logement ou du départ — la réponse est non : proposer « Itinéraire jusqu'au
    /// départ » sans savoir où aller vaut moins que de ne rien proposer.
    pub fn starts_far(&self, trail: &TrailRow) -> bool {
        self.start_metres(trail)
            .is_some_and(|metres| metres >= crate::config::FAR_START_METRES)
    }

    /// Le départ est-il le logement lui-même ?
    pub fn starts_at_property(&self, trail: &TrailRow) -> bool {
        self.start_metres(trail)
            .is_some_and(|metres| metres < crate::config::AT_PROPERTY_METRES)
    }
}
