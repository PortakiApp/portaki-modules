//! Ce que les appareils savent d'eux-mêmes : la vue du livret, le mode d'emploi, l'emoji proposé.

mod device;
mod display;
mod guide;
mod howto;
mod suggest;

pub use guide::{Appliance, Guide};
pub use howto::{extract_howto_steps, is_video_url, HowToStep};
