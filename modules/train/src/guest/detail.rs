//! Guest explore detail — from/to header, destination filter, next departures.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{Emphasis, SurfaceLevel};
use portaki_sdk::sdui::primitives::{
    Card, FilterBar, FilterChip, KeyValue, Stack, Text, TimedEntry,
};
use portaki_sdk::sdui::surface::Surface;

use crate::content::{schedule_for, station_caption, DESTINATIONS};

#[derive(Serialize)]
struct DestParams<'a> {
    dest: &'a str,
}

pub fn build_detail_page(_ctx: &GuestContext, selected: &str) -> Surface {
    Surface::new(Stack::new().gap(12.0).children(vec![
            Component::Card(from_to_card(selected)),
            Component::FilterBar(destination_filter_bar(selected)),
            Component::Text(
                Text::new()
                    .text("i18n:explore.detail.upcoming")
                    .variant(TextVariant::Title),
            ),
            Component::Card(schedule_card(selected)),
            Component::Text(
                Text::new()
                    .text("i18n:explore.detail.disclaimer")
                    .variant(TextVariant::Caption)
                    .emphasis(Emphasis::Subtle),
            ),
        ]))
    .with_id(crate::guest::EXPLORE_DETAIL)
}

fn from_to_card(selected: &str) -> Card {
    Card::new().surface(SurfaceLevel::Elevated).children(vec![
        Component::KeyValue(
            KeyValue::new()
                .key("i18n:explore.detail.from")
                .value(station_caption()),
        ),
        Component::KeyValue(
            KeyValue::new()
                .key("i18n:explore.detail.to")
                .value(selected),
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
    let action = Action::navigate(
        NavigateTarget::path("train"),
        Some(json_value(DestParams { dest: destination })),
    );

    Component::FilterChip(
        FilterChip::new()
            .label(destination)
            .selected(is_selected)
            .action(action),
    )
}

fn schedule_card(selected: &str) -> Card {
    let children = schedule_for(selected)
        .into_iter()
        .map(|departure| {
            Component::TimedEntry(
                TimedEntry::new()
                    .time(departure.time)
                    .title(selected)
                    .subtitle(format!("{} · {}", departure.platform, departure.note)),
            )
        })
        .collect();
    Card::new()
        .surface(SurfaceLevel::Elevated)
        .children(children)
}
