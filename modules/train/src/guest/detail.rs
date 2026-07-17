//! Guest explore detail — from/to header, destination filter, next departures.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{Emphasis, SurfaceLevel};
use portaki_sdk::sdui::primitives::{
    Card, FilterBar, FilterChip, KeyValue, Stack, Text, TimedEntry,
};
use portaki_sdk::sdui::surface::Surface;
use serde_json::json;

use crate::content::{schedule_for, station_caption, DESTINATIONS};

pub fn build_detail_page(ctx: &GuestContext, selected: &str) -> Surface {
    Surface::new(Stack::new().gap(json!(12)).children(vec![
            Component::Card(from_to_card(ctx, selected)),
            Component::FilterBar(destination_filter_bar(selected)),
            Component::Text(
                Text::new()
                    .text(json!("i18n:explore.detail.upcoming"))
                    .variant(json!("title")),
            ),
            Component::Card(schedule_card(selected)),
            Component::Text(
                Text::new()
                    .text(json!("i18n:explore.detail.disclaimer"))
                    .variant(json!("caption"))
                    .emphasis(Emphasis::Subtle),
            ),
        ]))
    .with_id("explore.detail")
}

fn from_to_card(ctx: &GuestContext, selected: &str) -> Card {
    Card::new().surface(SurfaceLevel::Elevated).children(vec![
        Component::KeyValue(
            KeyValue::new()
                .key(json!("i18n:explore.detail.from"))
                .value(json!(station_caption(&ctx.locale))),
        ),
        Component::KeyValue(
            KeyValue::new()
                .key(json!("i18n:explore.detail.to"))
                .value(json!(selected)),
        ),
    ])
}

fn destination_filter_bar(selected: &str) -> FilterBar {
    let chips = DESTINATIONS
        .iter()
        .map(|destination| destination_chip(destination, *destination == selected))
        .collect();
    FilterBar::new().children(chips)
}

fn destination_chip(destination: &str, is_selected: bool) -> Component {
    let action = serde_json::to_value(Action::Navigate {
        to: "train".to_string(),
        params: Some(json!({ "dest": destination })),
    })
    .unwrap_or(json!({}));

    Component::FilterChip(
        FilterChip::new()
            .label(json!(destination))
            .selected(json!(is_selected))
            .action(action),
    )
}

fn schedule_card(selected: &str) -> Card {
    let children = schedule_for(selected)
        .into_iter()
        .map(|departure| {
            Component::TimedEntry(
                TimedEntry::new()
                    .time(json!(departure.time))
                    .title(json!(selected))
                    .subtitle(json!(format!(
                        "{} · {}",
                        departure.platform, departure.note
                    ))),
            )
        })
        .collect();
    Card::new()
        .surface(SurfaceLevel::Elevated)
        .children(children)
}
