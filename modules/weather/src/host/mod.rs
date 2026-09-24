//! Host dashboard surfaces — config cards embedded in the module sheet.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Form, Page, Select, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::load_config;
use crate::entities::WeatherUnits;

/// Host configuration surface (units + refresh cadence).
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::CloudSun
)]
pub fn render_host_main(ctx: HostContext) -> Surface {
    let _ = ctx;
    let config = load_config().unwrap_or_default();

    let units_value = match config.units {
        WeatherUnits::Celsius => "celsius",
        WeatherUnits::Fahrenheit => "fahrenheit",
    };

    let form_children: Vec<Component> = vec![
        Card::new()
            .title("i18n:host.section.display")
            .subtitle("i18n:host.section.display.help")
            .icon(IconName::CloudSun)
            .children(vec![
                Field::new()
                    .name("units")
                    .label("i18n:host.units.label")
                    .child(
                        Select::new()
                            .name("units")
                            .options(vec![
                                ChoiceOption::new("celsius", "°C"),
                                ChoiceOption::new("fahrenheit", "°F"),
                            ])
                            .value(units_value),
                    )
                    .into(),
                Field::new()
                    .name("refresh_interval")
                    .label("i18n:host.refresh.label")
                    .child(
                        Select::new()
                            .name("refresh_interval")
                            .options(vec![
                                ChoiceOption::new("1h", "i18n:host.hourly"),
                                ChoiceOption::new("3h", "i18n:host.3hours"),
                                ChoiceOption::new("6h", "i18n:host.6hours"),
                            ])
                            .value(config.refresh_interval),
                    )
                    .into(),
            ])
            .into(),
        Text::new()
            .text("i18n:host.main.help")
            .variant(TextVariant::Caption)
            .into(),
    ];

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Surface::new(Page::new().child(Form::new().children(form_children)))
        .with_id(crate::ids::HOST_MAIN)
}
