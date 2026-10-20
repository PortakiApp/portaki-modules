//! Les opérations : le catalogue de départ, la demande du voyageur, la réponse de l'hôte, les
//! lectures, les tâches et la tuile de statistiques — une par fichier, à côté de leurs aides
//! (le stockage des demandes, le séjour visé, les tâches).

pub mod amenities_list;
pub mod earlier;
pub mod list_for_stay;
pub mod list_open_count;
pub mod list_recent;
pub mod reports;
pub mod seed_defaults;
pub mod stats_summary;
mod stay;
pub mod submit;
pub mod task_complete;
pub mod task_toggle;
mod tasks;
pub mod timeline_tasks;
pub mod update_status;

pub use amenities_list::amenities_list;
pub use list_for_stay::{list_for_stay, ConsumableReportRow, ListForStayArgs};
pub use list_open_count::{list_open_count, OpenCountDto};
pub use list_recent::list_recent;
pub use seed_defaults::{seed_defaults, SEED_DEFAULTS};
pub use stats_summary::stats_summary;
pub use submit::{submit, SubmitArgs, SUBMIT};
pub use task_complete::task_complete;
pub use task_toggle::task_toggle;
pub use timeline_tasks::timeline_tasks;
pub use update_status::{update_status, UpdateStatusArgs, UPDATE_STATUS};
