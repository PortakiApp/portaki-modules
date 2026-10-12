//! Host dashboard surface — the Wi-Fi settings drawer (spec Wi-Fi §2).
//!
//! Two `Section`s — the shell's two cards — in the spec's order: the networks (one to three), and
//! what the guest sees. The drawer chrome (title, switch, Publier) belongs to the dashboard. A rule
//! broken shows under its field (`Field::error`), the same one `publishReadiness` blocks on.
//!
//! A choice that changes the form (a network's security, the reveal) is sent back as it is made
//! (`emitOnChange`) and read from the draft first: `visibleWhen` only says `name=value`, and the
//! password has to hide on one value of three.

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    ChoiceList, Field, FieldHint, Form, InlineNotice, Link, NumberInput, Page, SecretInput,
    Section, Stack, StepList, Text, TextArea, TextInput, Toggle, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::{
    ModuleConfig, Network, RevealPolicy, WifiSecurity, DEFAULT_REVEAL_HOURS, MAX_NETWORKS,
};
use crate::reveal::{Schedule, Source};

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
        .child(display_card(&config, &Schedule::load(&config), &ctx));
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
        .hint(counter(networks.len()))
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
    Section::new()
        .title("i18n:host.section.networks")
        .subtitle("i18n:host.networks.help")
        .children(children)
        .into()
}

/// « 2 / 3 » : how many networks of the three allowed.
fn counter(count: usize) -> String {
    let (count, max) = (count.to_string(), MAX_NETWORKS.to_string());
    match t!("host.networks.count", count = &count, max = &max) {
        Ok(text) if text.contains(&count) => text,
        _ => format!("{count} / {max}"),
    }
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

    // Le choix en cours d'abord : changer la sécurité re-rend le tiroir avec ce choix.
    let security = ctx
        .input_str(&name("security"))
        .and_then(parse_security)
        .unwrap_or(network.security);

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
            "security",
            "i18n:config.security",
            security_choice_list(&name("security"), security).into(),
        )
        .into(),
    ];
    // Un seul réseau n'a pas besoin de nom : le libellé n'apparaît qu'à partir de deux (§3). Masqué,
    // il n'est pas envoyé, et la fusion de `updateConfig` garde ce qui était enregistré.
    if several {
        let saved = network
            .label
            .as_ref()
            .map(|l| l.host_value(ctx).to_string())
            .filter(|l| !l.trim().is_empty());
        // « Principal » pour le premier, que l'hôte n'a jamais eu à nommer tant qu'il était seul.
        let value = saved
            .or_else(|| {
                (index == 0)
                    .then(|| t!("host.label.default").ok())
                    .flatten()
                    .filter(|text| text != "host.label.default")
            })
            .unwrap_or_default();
        children.push(
            field(
                "label",
                "i18n:host.label.label",
                TextInput::new()
                    .name(name("label"))
                    .value(value)
                    .placeholder("i18n:host.label.placeholder")
                    .into(),
            )
            .required(true)
            .into(),
        );
    }
    if security != WifiSecurity::Nopass {
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
fn display_card(config: &ModuleConfig, schedule: &Schedule, ctx: &HostContext) -> Component {
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
    let mut children: Vec<Component> = vec![
        ToggleRow::new()
            .name("show_qr")
            .label("i18n:host.showQr.label")
            .description("i18n:host.showQr.help")
            .checked(config.show_qr)
            .into(),
        note.into(),
    ];
    children.extend(reveal_fields(config, schedule, ctx));
    Section::new()
        .title("i18n:host.section.display")
        .children(children)
        .into()
}

/// Révélation : celle d'Accès en lecture seule quand il la partage, sinon le choix du Wi-Fi.
fn reveal_fields(config: &ModuleConfig, schedule: &Schedule, ctx: &HostContext) -> Vec<Component> {
    if let Source::Peer {
        module_id,
        display_name,
    } = &schedule.source
    {
        let policy = policy_label(schedule);
        let line = match t!("host.reveal.inherited", policy = &policy) {
            Ok(text) if text.contains(&policy) => text,
            _ => policy.clone(),
        };
        let name = if display_name.trim().is_empty() {
            module_id.as_str()
        } else {
            display_name.as_str()
        };
        let edit = match t!("host.reveal.editIn", module = name) {
            Ok(text) if text.contains(name) => text,
            _ => name.to_string(),
        };
        return vec![Field::new()
            .label("i18n:host.section.reveal")
            .children(vec![
                Text::new().text(line).variant(TextVariant::Body).into(),
                // La page de réglages du module qui le partage, dans la coque : son id vient de la
                // plateforme (`ModulePeer`), jamais d'ici.
                Link::new()
                    .label(edit)
                    .href(format!("/listings/{}/modules/{module_id}", ctx.property_id))
                    .into(),
            ])
            .into()];
    }
    let policy = ctx
        .input_str("reveal_policy")
        .and_then(parse_reveal)
        .unwrap_or(config.reveal_policy);
    let mut policy_field = Field::new()
        .name("reveal_policy")
        .label("i18n:host.section.reveal")
        .children(vec![
            reveal_choice_list(policy).into(),
            FieldHint::new()
                .text("i18n:host.section.reveal.help")
                .into(),
        ]);
    if let Some(error) = config.error_of("config.reveal_policy", &ctx.locale) {
        policy_field = policy_field.error(error);
    }
    let mut fields = vec![policy_field.into()];
    if policy == RevealPolicy::Custom {
        let mut hours = Field::new()
            .name("reveal_hours")
            .label("i18n:host.reveal.hours")
            .visibleWhen("reveal_policy=custom")
            .child(
                NumberInput::new()
                    .name("reveal_hours")
                    .value(f64::from(
                        config.reveal_hours.unwrap_or(DEFAULT_REVEAL_HOURS),
                    ))
                    .min(1.0)
                    .max(168.0),
            );
        if let Some(error) = config.error_of("config.reveal_hours", &ctx.locale) {
            hours = hours.error(error);
        }
        fields.push(hours.into());
    }
    fields
}

/// « La veille à 16 h », « 6 h avant l'arrivée » : the option as the host chose it.
fn policy_label(schedule: &Schedule) -> String {
    if schedule.policy == RevealPolicy::Custom {
        let hours = schedule.hours.to_string();
        return match t!("host.reveal.customHours", hours = &hours) {
            Ok(text) if text.contains(&hours) => text,
            _ => format!("{hours} h"),
        };
    }
    let key = format!("host.reveal.{}", schedule.policy.i18n_key());
    t!(&key).unwrap_or(key)
}

fn parse_reveal(raw: &str) -> Option<RevealPolicy> {
    RevealPolicy::ALL
        .iter()
        .copied()
        .find(|p| p.as_wire() == raw.trim())
}

fn parse_security(raw: &str) -> Option<WifiSecurity> {
    [WifiSecurity::Wpa, WifiSecurity::Wep, WifiSecurity::Nopass]
        .into_iter()
        .find(|s| s.as_wire() == raw.trim())
}

/// Trois choix et non quatre : WPA couvre WPA2 et WPA3, que les téléphones lisent de la même façon.
fn security_choice_list(name: &str, security: WifiSecurity) -> ChoiceList {
    ChoiceList::new()
        .name(name)
        .value(security.as_wire())
        .emitOnChange(true)
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
        .emitOnChange(true)
        .choices(
            RevealPolicy::ALL
                .iter()
                .map(|p| {
                    let key = p.i18n_key();
                    ChoiceOption::new(p.as_wire(), format!("i18n:host.reveal.{key}"))
                        .description(format!("i18n:host.reveal.{key}.desc"))
                        .icon(if *p == RevealPolicy::AtCheckin {
                            IconName::Key
                        } else {
                            IconName::ClockCircle
                        })
                })
                .collect(),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use portaki_test_utils::MockContext;

    /// Spec §3 : Accès partage son calendrier → une ligne en lecture seule et le lien vers ses
    /// réglages, pas de choix propre au Wi-Fi.
    #[test]
    #[serial_test::serial]
    fn a_shared_schedule_is_read_only_with_a_link_to_its_module() {
        let config = ModuleConfig::default();
        let peer = crate::reveal::Peer {
            module_id: "access-guide".into(),
            display_name: "Accès et parking".into(),
            shared: serde_json::json!({ "reveal_policy": "day_before_16h" })
                .as_object()
                .cloned()
                .unwrap(),
        };
        let schedule = Schedule::resolve(&config, Some(&peer));
        let json = MockContext::host()
            .with_translation("host.reveal.dayBefore16h", "La veille à 16 h")
            .with_translation(
                "host.reveal.inherited",
                "{policy} · comme les codes d'accès",
            )
            .with_translation("host.reveal.editIn", "Modifier dans {module}")
            .run(|ctx| {
                let property = ctx.property_id;
                let fields = reveal_fields(&config, &schedule, &ctx);
                (serde_json::to_string(&fields).unwrap(), property)
            });
        let (json, property) = json;
        assert!(
            json.contains("La veille à 16 h · comme les codes d'accès"),
            "{json}"
        );
        assert!(json.contains("Modifier dans Accès et parking"), "{json}");
        assert!(
            json.contains(&format!("/listings/{property}/modules/access-guide")),
            "{json}"
        );
        assert!(!json.contains(r#""name":"reveal_policy""#), "{json}");

        // Sans pair : le choix du Wi-Fi.
        let own = MockContext::host().run(|ctx| {
            serde_json::to_string(&reveal_fields(
                &config,
                &Schedule::resolve(&config, None),
                &ctx,
            ))
            .unwrap()
        });
        assert!(own.contains(r#""name":"reveal_policy""#), "{own}");
    }
}
