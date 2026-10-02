//! Le bloc que ce module ajoute aux e-mails du jour d'arrivée et du lendemain.
//!
//! Les horaires du jour, trois équipements au plus. La semaine entière reste sur la page de séjour,
//! où le bloc renvoie : un e-mail n'a pas à porter sept lignes par équipement.

use chrono::Datelike;
use portaki_sdk::email::{EmailBlock, EmailBlocks, EmailContextArgs};
use portaki_sdk::host::time::{self, PropertyTz};
use portaki_sdk::prelude::*;

use crate::config::ModuleConfig;

/// Ancrage de la section du module sur la page de séjour ; la plateforme construit l'URL.
const ANCHOR: &str = "facility-hours";

/// Ce que le design laisse à une ligne de liste : au-delà, la valeur serait coupée.
const ROWS: usize = 3;

#[portaki_sdk::email_blocks(ArrivalDay | PostArrival => [List])]
pub fn email_blocks(ctx: Context, _args: EmailContextArgs) -> Result<EmailBlocks> {
    let config = ModuleConfig::load(&ctx)?;
    let now = time::now()?;
    let tz = PropertyTz::parse(&ctx.timezone);
    // Le jour au fuseau du logement : « aujourd'hui » se compte là où le voyageur se trouve.
    let today = tz
        .as_ref()
        .map(|tz| tz.to_local(now).naive_local().weekday())
        .unwrap_or_else(|| now.naive_utc().weekday());

    let mut block = EmailBlock::list(t!("email.block.label")?).title(t!("email.block.title")?);
    let mut rows = 0;
    for facility in &config.facilities {
        if rows == ROWS {
            break;
        }
        let schedule = facility.schedule();
        // Sans heures structurées, la phrase de l'hôte reste sur la page de séjour : elle est
        // souvent plus longue qu'une ligne de liste, et la couper la rendrait fausse.
        let hours = if schedule.all_day {
            t!("email.state.open")?
        } else if let Some(span) = schedule.span_on(today) {
            format!("{} – {}", minutes(span.opens), minutes(span.closes))
        } else {
            t!("email.state.closed")?
        };
        let title = facility.title.get(&ctx.locale).trim().to_string();
        if title.is_empty() {
            continue;
        }
        block = block.row(title, hours);
        rows += 1;
    }
    if rows == 0 {
        return Ok(EmailBlocks::new());
    }

    Ok(EmailBlocks::new().with(block.link(t!("email.block.link")?, ANCHOR)))
}

fn minutes(value: u32) -> String {
    format!("{:02}:{:02}", value / 60, value % 60)
}
