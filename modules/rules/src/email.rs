//! Les premières règles dans les e-mails de séjour de Portaki.

use portaki_sdk::prelude::*;

use crate::Rules;

#[portaki_sdk::email_vars(StayLink | Arrival | PostArrival => [HouseRulesTeaser])]
pub fn email_vars(ctx: Context, _args: EmailContextArgs) -> Result<EmailVars> {
    let lines: Vec<String> = Rules::load(&ctx)?
        .rows()
        .iter()
        .take(3)
        .map(|r| match r.detail.for_ctx(&ctx) {
            "" => r.title.for_ctx(&ctx).to_string(),
            detail => format!("{} — {detail}", r.title.for_ctx(&ctx)),
        })
        .collect();
    Ok(EmailVars::new().with(EmailVar::HouseRulesTeaser, lines.join("\n")))
}
