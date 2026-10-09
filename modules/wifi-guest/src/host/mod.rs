//! Host dashboard surface — the Wi-Fi settings drawer (spec Wi-Fi §2).
//!
//! Two cards, in the spec's order: the networks (one to three), and what the guest sees. The
//! drawer chrome (title, switch, Publier) belongs to the dashboard. A rule broken shows under its
//! field (`Field::error`), the same one `publishReadiness` blocks on.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Card, ChoiceList, Field, FieldHint, Form, InlineNotice, Page, SecretInput, Stack, StepList,
    Text, TextArea, TextInput, Toggle, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{ModuleConfig, Network, RevealPolicy, WifiSecurity, MAX_NETWORKS};

mod stay;

pub use stay::render_host_stay;

#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    label_key = "catalog.host.main",
    icon = IconName::Wifi
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let mut networks = config.networks();
    let count = rows_count(&ctx, networks.len());
    networks.resize_with(count, Network::default);

    let form = Form::new()
        .child(networks_card(&config, &networks, &ctx))
        .child(display_card(&config, &ctx));
    Ok(Surface::new(Page::new().child(form)).with_id(MAIN))
}

/// Rows drawn: the saved ones, one more after « Ajouter un réseau », never more than three, at
/// least one (spec §2.1 « défaut : 1 ligne »).
fn rows_count(ctx: &HostContext, saved: usize) -> usize {
    ctx.input_u64("networks_count")
        .map(|n| n as usize)
        .unwrap_or(saved)
        .clamp(1, MAX_NETWORKS)
}

#[derive(Serialize)]
struct NetworksCountInput {
    networks_count: usize,
}

/// §2.1 Réseaux.
fn networks_card(config: &ModuleConfig, networks: &[Network], ctx: &HostContext) -> Component {
    let several = networks.len() > 1;
    let rows: Vec<Component> = networks
        .iter()
        .enumerate()
        .map(|(index, network)| network_row(config, index, network, several, ctx))
        .collect();
    let mut list = StepList::new()
        .label("i18n:host.networks.label")
        .hint("i18n:host.networks.help")
        .removeLabel("i18n:host.networks.remove")
        .itemKeyPrefix("networks")
        .children(rows);
    if networks.len() < MAX_NETWORKS {
        list = list
            .addLabel("i18n:host.networks.add")
            .addAction(Action::emit(
                contracts::shell::SURFACE_INPUT,
                Some(json_value(NetworksCountInput {
                    networks_count: networks.len() + 1,
                })),
            ));
    }
    let mut children: Vec<Component> = vec![list.into()];
    if networks.len() >= MAX_NETWORKS {
        children.push(
            Text::new()
                .text("i18n:host.networks.max")
                .variant(TextVariant::Caption)
                .into(),
        );
    }
    if let Some(error) = config.error_of("config.networks", &ctx.locale) {
        children.push(InlineNotice::new().tone(Tone::Danger).message(error).into());
    }
    Card::new()
        .title("i18n:host.section.networks")
        .children(children)
        .into()
}

fn network_row(
    config: &ModuleConfig,
    index: usize,
    network: &Network,
    several: bool,
    ctx: &HostContext,
) -> Component {
    let name = |key: &str| format!("networks.{index}.{key}");
    let error =
        |key: &str| config.error_of(&format!("config.networks[{index}].{key}"), &ctx.locale);
    let field = |key: &str, label: &str, child: Component| {
        let mut field = Field::new().name(name(key)).label(label).child(child);
        if let Some(error) = error(key) {
            field = field.error(error);
        }
        field
    };
    let id = if network.id.trim().is_empty() {
        format!("network-{}", index + 1)
    } else {
        network.id.clone()
    };

    let mut children: Vec<Component> = vec![
        // L'id garde le mot de passe d'une ligne d'un enregistrement à l'autre : sans lui, la
        // plateforme rapproche par position, et réordonner donnerait le mot de passe au voisin.
        TextInput::new().name(name("id")).value(id).into(),
        field(
            "ssid",
            "i18n:host.ssid.label",
            Stack::new()
                .children(vec![
                    FieldHint::new().text("i18n:host.ssid.desc").into(),
                    TextInput::new()
                        .name(name("ssid"))
                        .value(network.ssid.clone())
                        .placeholder("i18n:host.ssid.placeholder")
                        .into(),
                ])
                .into(),
        )
        .required(true)
        .into(),
        field(
            "label",
            "i18n:host.label.label",
            TextInput::new()
                .name(name("label"))
                .value(
                    network
                        .label
                        .as_ref()
                        .map(|l| l.host_value(ctx).to_string())
                        .unwrap_or_default(),
                )
                .placeholder("i18n:host.label.placeholder")
                .into(),
        )
        .required(several)
        .into(),
        field(
            "security",
            "i18n:config.security",
            security_choice_list(&name("security"), network.security).into(),
        )
        .into(),
    ];
    if network.security != WifiSecurity::Nopass {
        children.push(
            field(
                "password",
                "i18n:host.password.label",
                SecretInput::new()
                    .name(name("password"))
                    .value(String::new())
                    .placeholder("i18n:host.password.placeholder")
                    .into(),
            )
            .into(),
        );
        if network.id == crate::config::LEGACY_NETWORK_ID {
            // Le réseau d'avant la liste : la plateforme n'a pas de ligne d'où garder son mot de
            // passe. Retapé, il est enregistré ; sinon « Publier » le réclame — le voyageur garde
            // l'ancien jusque-là.
            children.push(
                InlineNotice::new()
                    .tone(Tone::Warning)
                    .message("i18n:host.password.retype")
                    .into(),
            );
        }
    }
    children.push(
        field(
            "hidden",
            "i18n:host.hidden.label",
            Stack::new()
                .children(vec![
                    FieldHint::new().text("i18n:host.hidden.help").into(),
                    Toggle::new()
                        .name(name("hidden"))
                        .checked(network.hidden)
                        .label("i18n:host.hidden.toggle")
                        .into(),
                ])
                .into(),
        )
        .into(),
    );
    Stack::new()
        .id(format!("network-{index}"))
        .gap(10.0)
        .children(children)
        .into()
}

/// §2.2 Affichage.
fn display_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let mut note = Field::new()
        .name("note")
        .label("i18n:host.note.label")
        .required(false)
        .child(
            TextArea::new()
                .name("note")
                .value(
                    config
                        .note
                        .as_ref()
                        .map(|n| n.host_value(ctx).to_string())
                        .unwrap_or_default(),
                )
                .placeholder("i18n:host.note.placeholder"),
        );
    if let Some(error) = config.error_of("config.note", &ctx.locale) {
        note = note.error(error);
    }
    Card::new()
        .title("i18n:host.section.display")
        .children(vec![
            ToggleRow::new()
                .name("show_qr")
                .label("i18n:host.showQr.label")
                .description("i18n:host.showQr.help")
                .checked(config.show_qr)
                .into(),
            note.into(),
            Field::new()
                .name("reveal_policy")
                .label("i18n:host.section.reveal")
                .children(vec![
                    FieldHint::new()
                        .text("i18n:host.section.reveal.help")
                        .into(),
                    reveal_choice_list(config.reveal_policy).into(),
                ])
                .into(),
        ])
        .into()
}

/// Trois choix et non quatre : WPA couvre WPA2 et WPA3, que les téléphones lisent de la même façon.
fn security_choice_list(name: &str, security: WifiSecurity) -> ChoiceList {
    ChoiceList::new()
        .name(name)
        .value(security.as_wire())
        .choices(vec![
            ChoiceOption::new(WifiSecurity::Wpa.as_wire(), "i18n:config.security.wpa")
                .icon(IconName::Lock),
            ChoiceOption::new(WifiSecurity::Wep.as_wire(), "i18n:config.security.wep")
                .icon(IconName::Lock),
            ChoiceOption::new(
                WifiSecurity::Nopass.as_wire(),
                "i18n:config.security.nopass",
            )
            .icon(IconName::Wifi),
        ])
}

fn reveal_choice_list(policy: RevealPolicy) -> ChoiceList {
    ChoiceList::new()
        .name("reveal_policy")
        .value(policy.as_wire())
        .choices(vec![
            ChoiceOption::new(RevealPolicy::Always.as_wire(), "i18n:host.reveal.always")
                .description("i18n:host.reveal.always.desc")
                .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::HoursBefore24.as_wire(),
                "i18n:host.reveal.hoursBefore24",
            )
            .description("i18n:host.reveal.hoursBefore24.desc")
            .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::DayBefore16h.as_wire(),
                "i18n:host.reveal.dayBefore16h",
            )
            .description("i18n:host.reveal.dayBefore16h.desc")
            .icon(IconName::ClockCircle),
            ChoiceOption::new(
                RevealPolicy::AtCheckin.as_wire(),
                "i18n:host.reveal.atCheckin",
            )
            .description("i18n:host.reveal.atCheckin.desc")
            .icon(IconName::Key),
        ])
}
