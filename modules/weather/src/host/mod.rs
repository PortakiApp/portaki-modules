//! Host dashboard surfaces — config cards embedded in the module sheet.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::primitives::{Card, Field, Form, Page, Select, Text};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;
use crate::entities::WeatherUnits;

/// Host configuration surface (units + refresh cadence).
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::CloudSun
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;

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
                                ChoiceOption::new("celsius", "i18n:host.units.label.celsius"),
                                ChoiceOption::new("fahrenheit", "i18n:host.units.label.fahrenheit"),
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
                                ChoiceOption::new("1h", "i18n:host.refresh.label.1h"),
                                ChoiceOption::new("3h", "i18n:host.refresh.label.3h"),
                                ChoiceOption::new("6h", "i18n:host.refresh.label.6h"),
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
    Ok(Surface::new(Page::new().child(Form::new().children(form_children))).with_id(MAIN))
}
