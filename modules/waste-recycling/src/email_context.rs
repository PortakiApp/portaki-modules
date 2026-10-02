//! Le bloc que ce module ajoute à l'e-mail de lendemain d'arrivée (`post-arrival`).
//!
//! Un seul sujet : quand sort le bac, et ce que l'hôte en dit. Le reste — les bacs, ce qui va dans
//! lequel — reste sur la page de séjour, où le bloc renvoie.

use portaki_sdk::email::{EmailBlock, EmailBlocks, EmailContextArgs};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;

use crate::collection::{next_collection, Departure};
use crate::config::ModuleConfig;

/// Ancrage de la section du module sur la page de séjour ; la plateforme construit l'URL.
const ANCHOR: &str = "waste-recycling";

#[portaki_sdk::email_blocks(PostArrival => [Info])]
pub fn email_blocks(ctx: Context, _args: EmailContextArgs) -> Result<EmailBlocks> {
    let config = ModuleConfig::load(&ctx)?;
    if config.is_empty() {
        return Ok(EmailBlocks::new());
    }

    // Sans jour coché, il n'y a rien à annoncer : la phrase de l'hôte seule ferait un bloc qui ne
    // dit pas quand agir, et le livret la porte déjà.
    let now = time::now()?;
    let tz = PropertyTz::parse(&ctx.timezone);
    let Some(next) = next_collection(
        &config.collection_days(),
        now,
        tz.as_ref(),
        ctx.stay.as_ref().and_then(|stay| stay.checkout_at),
    ) else {
        return Ok(EmailBlocks::new());
    };

    let title = if next.today {
        t!("email.collection.today")?
    } else {
        t!(
            "email.collection.next",
            day = time::weekday_name(next.day, &ctx.locale)
        )?
    };

    // Ce que le départ change passe devant la phrase de l'hôte : c'est le seul moment où le
    // voyageur doit agir avant de partir.
    let text = match next.departure {
        Some(Departure::SameDay) => t!("email.collection.departureDay")?,
        Some(Departure::Eve) => t!("email.collection.departureEve")?,
        None => {
            let note = config.takeout_note.get(&ctx.locale).trim().to_string();
            if note.is_empty() {
                config.collection_schedule.get(&ctx.locale).trim().to_string()
            } else {
                note
            }
        }
    };
    if text.is_empty() {
        return Ok(EmailBlocks::new());
    }

    Ok(EmailBlocks::new().with(
        EmailBlock::info(t!("email.block.label")?, text)
            .title(title)
            .link(t!("email.block.link")?, ANCHOR),
    ))
}
