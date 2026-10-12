//! Host dashboard surfaces — design `editorNuki` / `nuki-editor-v1`, and the stay encart.

mod regenerate;
mod stay;

pub use regenerate::render_host_regenerate;
pub use stay::render_host_stay;

use portaki_sdk::prelude::*;
use portaki_sdk::sdui::common::Tone;
use portaki_sdk::sdui::primitives::{
    Card, Field, FieldHint, Form, InlineNotice, Page, SecretInput, Stack, TextInput, ToggleRow,
};
use portaki_sdk::sdui::surface::Surface;

use crate::config::ModuleConfig;

// « Serrure hors ligne », « Batterie faible » dans À venir : `crate::tasks`.
#[portaki_sdk::nav(
    placement = HostPlacement::WorkspaceTimelineTask,
    path = "tasks",
    label_key = "catalog.host.tasks",
    icon = IconName::DangerTriangle
)]
#[portaki_sdk::surface(
    host,
    id = "main",
    placement = HostPlacement::PropertyModuleSheet,
    design_id = DesignId::NukiEditorV1,
    label_key = "catalog.host.main",
    icon = IconName::Lock
)]
pub fn render_host_main(ctx: HostContext) -> Result<Surface> {
    let config = ModuleConfig::load(&ctx)?;
    let form_children: Vec<Component> = vec![lock_card(&config, &ctx), codes_card(&config, &ctx)];

    // No Page title / Save — the modules sheet owns chrome + footer Save.
    Ok(Surface::new(
        Page::new().child(Form::new().child(Stack::new().gap(16.0).children(form_children))),
    )
    .with_id(MAIN))
}

/// Le champ, avec le message de `problems` sous lui s'il y en a un.
fn named(config: &ModuleConfig, ctx: &HostContext, name: &str) -> Field {
    let field = Field::new().name(name);
    match config.error_of(name) {
        Some(error) => field.error(error.get(&ctx.locale).to_string()),
        None => field,
    }
}

/// §2.1 Serrure : la clé Nuki Web en lecture seule, l'identifiant de la serrure, son nom.
///
/// ponytail: l'identifiant se tape, il ne se choisit pas dans une liste — le catalogue de la
/// plateforme n'ouvre aucune opération « lister les serrures » (`GET /smartlock`). Pas de lien
/// « Gérer la connexion » non plus : aucune page de connexions que le tiroir puisse viser.
fn lock_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    let key = if crate::commands::has_nuki_byok(ctx) {
        "i18n:host.key.connected"
    } else {
        "i18n:host.key.missing"
    };
    Card::new()
        .title("i18n:host.section.lock")
        .icon(IconName::Lock)
        .children(vec![
            // Lu, pas saisi : sans `name`, le formulaire ne l'envoie pas.
            InlineNotice::new().tone(Tone::Neutral).message(key).into(),
            Field::new()
                .name("smartlock_id")
                .label("i18n:host.smartlockId.label")
                .child(
                    TextInput::new()
                        .name("smartlock_id")
                        .value(config.smartlock_id.clone()),
                )
                .child(FieldHint::new().text("i18n:host.smartlockId.hint"))
                .into(),
            named(config, ctx, "device_name")
                .label("i18n:host.deviceName.label")
                .required(false)
                .child(
                    TextInput::new()
                        .name("device_name")
                        .value(config.device_name.clone())
                        .placeholder("i18n:host.deviceName.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.deviceName.hint"))
                .into(),
        ])
        .into()
}

/// §2.2 Codes : le code clavier commun, et un code par séjour — éteint par défaut, contre la
/// spec, avec l'avertissement toujours visible : écrire sur la serrure se décide.
fn codes_card(config: &ModuleConfig, ctx: &HostContext) -> Component {
    Card::new()
        .title("i18n:host.section.codes")
        .icon(IconName::Lock)
        .children(vec![
            named(config, ctx, "keypad_code")
                .label("i18n:host.keypadCode.label")
                .required(false)
                .child(
                    SecretInput::new()
                        .name("keypad_code")
                        // Never sent back: blank keeps the saved code.
                        .value(String::new())
                        .placeholder("i18n:host.keypadCode.placeholder"),
                )
                .child(FieldHint::new().text("i18n:host.keypadCode.hint"))
                .into(),
            ToggleRow::new()
                .name("code_per_stay")
                .label("i18n:host.codePerStay.label")
                .description("i18n:host.codePerStay.help")
                .icon(IconName::Lock)
                .checked(config.code_per_stay)
                .into(),
            InlineNotice::new()
                .tone(Tone::Warning)
                .message("i18n:host.codePerStay.warning")
                .into(),
        ])
        .into()
}
