//! Le guide tel que le livret le lit : les appareils visibles, leurs textes dans la langue du
//! voyageur, et leur rangement par pièce.

use portaki_sdk::resource::rich_text;
use portaki_sdk::{Context, Result};

use super::howto::HowToStep;
use crate::{Device, Devices};

/// Un appareil visible, dans la langue du voyageur.
#[derive(Debug, Clone, Default)]
pub struct Appliance {
    pub id: String,
    pub name: String,
    pub emoji: String,
    /// Un document TipTap, ou du texte.
    pub description: String,
    pub featured: bool,
    pub location: String,
    pub model: String,
    pub safety_note: String,
    pub manual_url: String,
    pub manual_file: String,
    pub video_url: String,
    pub steps: Vec<HowToStep>,
}

impl Appliance {
    fn of(ctx: &Context, device: &Device) -> Self {
        let name = device.name.for_ctx(ctx).trim().to_string();
        Self {
            id: device.id.clone(),
            emoji: device.shown_emoji(&name),
            description: device.description.for_ctx(ctx).to_string(),
            featured: device.featured,
            location: device.location.for_ctx(ctx).trim().to_string(),
            model: device.model.clone(),
            safety_note: device.safety_note.for_ctx(ctx).to_string(),
            manual_url: device.manual_url.clone(),
            manual_file: device.manual_file.clone(),
            video_url: device.video_url.clone(),
            steps: device
                .steps
                .iter()
                .filter(|step| !step.text.for_ctx(ctx).trim().is_empty())
                .map(|step| HowToStep {
                    text: step.text.for_ctx(ctx).trim().to_string(),
                    image: Some(step.image.trim().to_string()).filter(|i| !i.is_empty()),
                })
                .collect(),
            name,
        }
    }

    /// Un mode d'emploi, sous une forme ou une autre : étapes, texte, PDF ou lien (§9 #4).
    pub fn has_manual(&self) -> bool {
        !self.steps.is_empty()
            || !rich_text::plain_text(&self.description).trim().is_empty()
            || !self.manual_file.trim().is_empty()
            || !self.manual_url.trim().is_empty()
    }
}

/// Les appareils visibles et ce qui vaut pour tous.
pub struct Guide {
    /// Actifs et nommés, dans l'ordre de l'hôte.
    pub devices: Vec<Appliance>,
    pub paper_manuals_location: String,
    limit: usize,
    room_order: Vec<String>,
}

impl Guide {
    pub fn load(ctx: &Context) -> Result<Self> {
        let devices = Devices::load(ctx)?;
        let meta = devices.meta();
        Ok(Self {
            devices: devices
                .rows()
                .iter()
                .filter(|d| !d.hidden)
                .map(|d| Appliance::of(ctx, d))
                .filter(|d| !d.name.is_empty())
                .collect(),
            paper_manuals_location: meta.paper_manuals_location.for_ctx(ctx).to_string(),
            limit: meta.limit(),
            room_order: meta.room_order.clone(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    pub fn find(&self, id: &str) -> Option<&Appliance> {
        self.devices.iter().find(|d| d.id == id)
    }

    /// Les mis en avant (au plus la limite de la carte) ; aucun : les premiers (spec §3, §9 #3).
    pub fn featured(&self) -> Vec<&Appliance> {
        let featured: Vec<&Appliance> = self
            .devices
            .iter()
            .filter(|d| d.featured)
            .take(self.limit)
            .collect();
        if featured.is_empty() {
            self.devices.iter().take(self.limit).collect()
        } else {
            featured
        }
    }

    /// Les appareils groupés par pièce (§2.4) : l'ordre choisi par l'hôte, puis celui d'apparition ;
    /// les appareils sans pièce en dernier, sous un groupe sans nom (« Autres »).
    pub fn by_room(&self) -> Vec<(Option<String>, Vec<&Appliance>)> {
        let mut rooms: Vec<(Option<String>, Vec<&Appliance>)> = Vec::new();
        let mut unplaced: Vec<&Appliance> = Vec::new();
        for device in &self.devices {
            if device.location.is_empty() {
                unplaced.push(device);
                continue;
            }
            match rooms
                .iter_mut()
                .find(|(name, _)| name.as_deref() == Some(device.location.as_str()))
            {
                Some((_, devices)) => devices.push(device),
                None => rooms.push((Some(device.location.clone()), vec![device])),
            }
        }
        rooms.sort_by_key(|(room, _)| self.rank(room.as_deref().unwrap_or_default()));
        if !unplaced.is_empty() {
            rooms.push((None, unplaced));
        }
        rooms
    }

    fn rank(&self, room: &str) -> usize {
        self.room_order
            .iter()
            .position(|r| r.trim() == room)
            .unwrap_or(usize::MAX)
    }
}
