//! `createChecklist` : une liste tirée d'un modèle.

use portaki_sdk::prelude::*;
use uuid::Uuid;

use super::templates;

/// Arguments for `createChecklist`.
#[portaki_sdk::wire]
#[portaki_sdk::params]
pub struct CreateChecklistArgs {
    /// One of [`templates::TEMPLATES`].
    pub template: String,
    /// The id the editor picked for the new list, so it opens selected. Already taken = nothing
    /// to do (a second click on the same card).
    #[serde(default)]
    pub id: Option<Uuid>,
}

#[portaki_sdk::command(
    name = "createChecklist",
    example(label = "Liste de départ", input = r#"{"template":"departure"}"#),
    example(
        label = "Ménage entre deux séjours",
        input = r#"{"template":"cleaning"}"#
    )
)]
pub fn create_checklist(_ctx: Context, args: CreateChecklistArgs) -> Result<()> {
    let template = templates::template(&args.template)
        .ok_or_else(|| PortakiError::Host(format!("unknown_template:{}", args.template)))?;
    if let Some(id) = args.id {
        if crate::domain::all_lists()?
            .iter()
            .any(|list| list.uuid() == id)
        {
            return Ok(());
        }
    }
    templates::add_from_template(template, args.id).map(|_| ())
}
