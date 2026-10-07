//! Shared guest SDUI body for access guide.

use portaki_sdk::host::i18n::{translate, Vars};
use portaki_sdk::prelude::*;
use portaki_sdk::sdui::action::Action;
use portaki_sdk::sdui::common::{
    BadgeSpec, KeyValueLayout, Leading, LeadingVisual, SecretState, Trailing, TrailingVisual,
};
use portaki_sdk::sdui::primitives::{
    Button, Eyebrow, Grid, InfoBanner, KeyValue, Link, ListItem, Map,
};

use crate::config::{
    BuildingAccess, DoorCodeTarget, MethodFields, ParkingLayer, PrimaryMethod, StaffKind,
};
use crate::reveal::SECRET_MASK;

use super::load::GuestData;

fn kind_label(kind: Option<&str>) -> String {
    match kind.map(str::trim).unwrap_or("") {
        "parking" => "i18n:host.step.kind.parking".into(),
        "door" => "i18n:host.step.kind.door".into(),
        "elevator" => "i18n:host.step.kind.elevator".into(),
        _ => "i18n:host.step.kind.other".into(),
    }
}

fn external_action(url: &str) -> Action {
    Action::external(url)
}

fn command_action(module_id: &ModuleId, name: OperationName, args: impl Serialize) -> Action {
    Action::command(module_id, name, args)
}

/// A link the guest may open: `https` only — never in clear, never `javascript:`.
fn is_https(url: &str) -> bool {
    url.starts_with("https://")
}

fn google_maps_search_url(lat: f64, lng: f64) -> String {
    format!("https://www.google.com/maps/search/?api=1&query={lat},{lng}")
}

/// Prefer parking plan URL, then in-person meeting GPS, then property GPS.
fn maps_url(data: &GuestData) -> Option<String> {
    if let Some(parking) = data.config.parking.as_ref() {
        let configured = parking.map_url.trim();
        if is_https(configured) {
            return Some(configured.to_string());
        }
    }
    if let Some((lat, lng)) = meeting_coords(data) {
        return Some(google_maps_search_url(lat, lng));
    }
    data.coordinates
        .map(|point| google_maps_search_url(point.lat, point.lng))
}

fn meeting_coords(data: &GuestData) -> Option<(f64, f64)> {
    match &data.config.method {
        MethodFields::InPerson {
            lat: Some(lat),
            lng: Some(lng),
            ..
        } => Some((*lat, *lng)),
        _ => None,
    }
}

/// Le plan de la carte d'accès, le logement nommé dessus.
///
/// <p>Le repère portait `"Logement"` — une chaîne française **écrite dans le Rust du module**, que
/// l'anglophone lisait telle quelle, et qui nommait autrement ce que la Carte du livret appelle
/// par son nom. Le nom du logement vient de la plateforme : il n'a pas à être traduit, et il est
/// le même partout.
fn map_at(name: &str, lat: f64, lng: f64) -> Component {
    let mut marker = MapMarker::new("property", lat, lng).kind(MapMarkerKind::Property);
    if !name.is_empty() {
        marker = marker.label(name);
    }
    Component::Map(
        Map::new()
            .viewport(MapViewport::new(lat, lng, Some(15.0)))
            .markers(vec![marker])
            .isStatic(true)
            .interactionMode(MapInteractionMode::None),
    )
}

fn property_map(data: &GuestData) -> Option<Component> {
    if let Some((lat, lng)) = meeting_coords(data) {
        return Some(map_at(&data.property_name, lat, lng));
    }
    data.coordinates
        .map(|point| map_at(&data.property_name, point.lat, point.lng))
}

fn kv_row(key_i18n: &str, value: &str, mono: bool) -> Component {
    let mut row = KeyValue::new().key(key_i18n).value(value);
    if mono {
        row = row.mono(true);
    }
    Component::KeyValue(row)
}

fn secret_display(data: &GuestData, plaintext: &str) -> String {
    if data.secrets_revealed {
        plaintext.to_string()
    } else {
        SECRET_MASK.to_string()
    }
}

fn push_secret_row(children: &mut Vec<Component>, data: &GuestData, key_i18n: &str, code: &str) {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return;
    }
    children.push(kv_row(key_i18n, &secret_display(data, trimmed), true));
}

/// Un code, en tuile : l'étiquette au-dessus, la valeur en grand dessous, et de quoi la copier.
///
/// C'est ce que le voyageur cherche sur la carte d'accueil — pas une ligne de tableau parmi
/// d'autres. La copie n'est offerte que sur un code révélé : copier un masque ne sert personne.
fn secret_tile(data: &GuestData, key_i18n: &str, icon: IconName, code: &str) -> Component {
    let mut tile = KeyValue::new()
        .key(key_i18n)
        .value(secret_display(data, code.trim()))
        .mono(true)
        .layout(KeyValueLayout::Tile)
        .icon(icon)
        .copy(data.secrets_revealed);
    // Masquée, la tuile dit qu'elle s'ouvrira et quand : sans cet état, le livret dessinait des
    // points sans rien promettre, et le voyageur croyait l'hôte en retard (§2.1).
    if !data.secrets_revealed {
        tile = tile.secret(SecretState::hidden(data.reveal_at_label.clone()));
    }
    Component::KeyValue(tile)
}

/// L'heure d'arrivée, en tuile à côté du code : les deux choses qu'on vérifie avant de sonner.
fn arrival_tile(data: &GuestData) -> Option<Component> {
    let hour = data.checkin_hour.as_deref()?;
    Some(Component::KeyValue(
        KeyValue::new()
            .key("i18n:guest.checkin")
            .value(hour.to_string())
            .layout(KeyValueLayout::Tile)
            .icon(IconName::Clock)
            .mono(true),
    ))
}

/// Les codes du moyen d'accès principal, en tuiles côte à côte.
///
/// Vide quand il n'y a rien à montrer : une grille d'une seule case vaudrait moins qu'une ligne.
fn secret_tiles(data: &GuestData) -> Vec<Component> {
    let mut tiles = Vec::new();
    if let Some(code) = data.config.keybox_code() {
        tiles.push(secret_tile(
            data,
            "i18n:guest.keybox.code",
            IconName::Key,
            code,
        ));
    }
    // Le code de secours d'une serrure connectée est un code comme un autre : il manquait aux
    // tuiles, et c'est la seule chose à composer quand le téléphone ne déverrouille pas.
    if let Some(code) = data.config.smart_lock_manual_code() {
        tiles.push(secret_tile(
            data,
            "i18n:guest.smartLock.manualCode",
            IconName::Lock,
            code,
        ));
    }
    if let Some(code) = data.config.gate_code() {
        tiles.push(secret_tile(
            data,
            "i18n:guest.building.gateCode",
            IconName::Lock,
            code,
        ));
    }
    tiles
}

fn push_text_row(children: &mut Vec<Component>, key_i18n: &str, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    children.push(kv_row(key_i18n, trimmed, false));
}

fn has_any_secret(data: &GuestData) -> bool {
    method_has_secret(&data.config.method)
        || data
            .config
            .building_access
            .as_ref()
            .and_then(|b| b.gate_code.as_deref())
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false)
        || data
            .config
            .parking
            .as_ref()
            .and_then(|p| p.code.as_deref())
            .map(|c| !c.trim().is_empty())
            .unwrap_or(false)
}

fn method_has_secret(method: &MethodFields) -> bool {
    match method {
        MethodFields::Keybox { code: Some(c), .. } => !c.trim().is_empty(),
        MethodFields::DoorCode { code, .. } => !code.trim().is_empty(),
        MethodFields::SmartLock {
            manual_code: Some(c),
            ..
        } => !c.trim().is_empty(),
        _ => false,
    }
}

fn door_target_key(target: DoorCodeTarget) -> &'static str {
    match target {
        DoorCodeTarget::Gate => "i18n:guest.doorCode.gate",
        DoorCodeTarget::Building => "i18n:guest.doorCode.building",
        DoorCodeTarget::Apartment => "i18n:guest.doorCode.apartment",
    }
}

fn staff_kind_key(kind: StaffKind) -> &'static str {
    match kind {
        StaffKind::Reception => "i18n:guest.buildingStaff.reception",
        StaffKind::Caretaker => "i18n:guest.buildingStaff.caretaker",
    }
}

#[portaki_sdk::wire(serialize)]
#[derive(Clone)]
struct SmartLockCommandArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    stay_id: Option<String>,
}

fn smart_lock_command_args(data: &GuestData) -> SmartLockCommandArgs {
    SmartLockCommandArgs {
        stay_id: data.stay_id.map(|id| id.to_string()),
    }
}

fn push_smart_lock_ctas(children: &mut Vec<Component>, data: &GuestData) {
    let Some(provider) = data
        .config
        .smart_lock_provider_module_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    if !data.secrets_revealed {
        return;
    }

    let args = smart_lock_command_args(data);
    children.push(Component::Button(
        Button::new()
            .label("i18n:guest.smartLock.unlock")
            .action(command_action(
                &ModuleId::new(provider),
                contracts::smart_lock::UNLOCK,
                args.clone(),
            )),
    ));
    children.push(Component::Button(
        Button::new()
            .label("i18n:guest.smartLock.getCredential")
            .variant(ButtonVariant::Outline)
            .action(command_action(
                &ModuleId::new(provider),
                contracts::smart_lock::GET_GUEST_CREDENTIAL,
                args,
            )),
    ));
}

fn push_method_instructions(children: &mut Vec<Component>, data: &GuestData) {
    if let Some(instructions) = data.texts.method_instructions.as_deref() {
        let trimmed = instructions.trim();
        if !trimmed.is_empty() {
            push_text_row(children, "i18n:guest.instructions", trimmed);
        }
    }
}

fn push_primary_method(children: &mut Vec<Component>, data: &GuestData, detailed: bool) {
    match &data.config.method {
        MethodFields::Keybox { location, code } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.keybox",
                false,
            ));
            push_text_row(children, "i18n:guest.keybox.location", location);
            if let Some(code) = code {
                push_secret_row(children, data, "i18n:guest.keybox.code", code);
            }
            if detailed {
                push_method_instructions(children, data);
            }
        }
        MethodFields::DoorCode { target, code } => {
            children.push(kv_row("i18n:guest.method", door_target_key(*target), false));
            push_secret_row(children, data, "i18n:guest.doorCode.code", code);
            if detailed {
                push_method_instructions(children, data);
            }
        }
        MethodFields::SmartLock { manual_code } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.smartLock",
                false,
            ));
            let has_provider = data
                .config
                .smart_lock_provider_module_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .is_some();
            if has_provider {
                push_smart_lock_ctas(children, data);
            }
            if let Some(manual_code) = manual_code {
                push_secret_row(
                    children,
                    data,
                    "i18n:guest.smartLock.manualCode",
                    manual_code,
                );
            }
            if detailed || !has_provider {
                push_method_instructions(children, data);
            }
        }
        MethodFields::InPerson {
            meeting_place,
            lat,
            lng,
            time_hint,
            contact,
        } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.inPerson",
                false,
            ));
            push_text_row(children, "i18n:guest.inPerson.meetingPlace", meeting_place);
            // Map + Open Maps use meeting GPS via property_map / maps_url when set.
            if let (Some(lat), Some(lng)) = (lat, lng) {
                children.push(kv_row(
                    "i18n:guest.inPerson.coords",
                    &format!("{lat:.5}, {lng:.5}"),
                    true,
                ));
            }
            if let Some(time_hint) = time_hint {
                push_text_row(children, "i18n:guest.inPerson.timeHint", time_hint);
            }
            if let Some(contact) = contact {
                push_text_row(children, "i18n:guest.inPerson.contact", contact);
            }
        }
        MethodFields::BuildingStaff {
            staff_kind,
            desk_location,
            hours,
            contact,
        } => {
            children.push(kv_row(
                "i18n:guest.method",
                staff_kind_key(*staff_kind),
                false,
            ));
            push_text_row(
                children,
                "i18n:guest.buildingStaff.deskLocation",
                desk_location,
            );
            if let Some(hours) = hours {
                push_text_row(children, "i18n:guest.buildingStaff.hours", hours);
            }
            if let Some(contact) = contact {
                push_text_row(children, "i18n:guest.buildingStaff.contact", contact);
            }
        }
        MethodFields::HostGreets {
            contact_note,
            eta_hint,
        } => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.hostGreets",
                false,
            ));
            if let Some(eta_hint) = eta_hint {
                push_text_row(children, "i18n:guest.hostGreets.etaHint", eta_hint);
            }
            if let Some(contact_note) = contact_note {
                push_text_row(children, "i18n:guest.hostGreets.contactNote", contact_note);
            }
        }
        MethodFields::Other {} => {
            children.push(kv_row(
                "i18n:guest.method",
                "i18n:guest.method.other",
                false,
            ));
            push_method_instructions(children, data);
        }
    }
}

fn push_building_access(
    children: &mut Vec<Component>,
    data: &GuestData,
    building: &BuildingAccess,
    detailed: bool,
) {
    let note = data
        .texts
        .building_note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if building.is_empty() && note.is_none() {
        return;
    }
    if let Some(gate) = building.gate_code.as_deref() {
        push_secret_row(children, data, "i18n:guest.building.gateCode", gate);
    }
    if let Some(intercom) = building.intercom.as_deref() {
        push_text_row(children, "i18n:guest.building.intercom", intercom);
    }
    if detailed {
        if let Some(note) = note {
            push_text_row(children, "i18n:guest.building.note", note);
        }
    }
}

fn push_parking(
    children: &mut Vec<Component>,
    data: &GuestData,
    parking: Option<&ParkingLayer>,
    _detailed: bool,
) {
    let info = data.texts.parking_info.trim();
    let code = parking.and_then(|p| p.code.as_deref());
    let has_layer = parking.is_some();
    if info.is_empty() && code.is_none() && !has_layer {
        return;
    }
    if parking.map(ParkingLayer::is_empty).unwrap_or(true) && info.is_empty() {
        return;
    }
    if !info.is_empty() {
        push_text_row(children, "i18n:guest.parking", info);
    }
    if let Some(code) = code {
        push_secret_row(children, data, "i18n:guest.parking.code", code);
    }
}

fn push_reveal_banner(children: &mut Vec<Component>, data: &GuestData) {
    if data.secrets_revealed || !has_any_secret(data) {
        return;
    }
    let Some(message) = data.reveal_locked_message.as_ref() else {
        return;
    };
    children.push(Component::InfoBanner(
        InfoBanner::new()
            .title("i18n:guest.reveal.lockedTitle")
            .message(message.clone()),
    ));
}

/// Une étape du chemin jusqu'à la porte.
///
/// Le rang va à gauche et le type à droite : ce sont des emplacements du `ListItem`, pas des
/// enfants. En enfants, le badge se dessinait dans le corps de la ligne au lieu de sa colonne, et
/// le voyageur perdait le fil de l'ordre — or une arrivée se suit dans l'ordre.
fn arrival_step(step: &crate::texts::StepText, rank: u32) -> ListItem {
    let mut item = ListItem::new()
        .title(step.title.trim())
        .leading(Leading::Visual(Box::new(LeadingVisual {
            index: Some(rank),
            ..LeadingVisual::default()
        })))
        .trailing(Trailing::Visual(Box::new(TrailingVisual {
            badge: Some(BadgeSpec {
                label: kind_label(step.kind.as_deref()),
                dot: true,
                ..BadgeSpec::default()
            }),
            ..TrailingVisual::default()
        })));
    if let Some(detail) = step.detail.as_ref() {
        let text = detail.trim();
        if !text.is_empty() {
            item = item.subtitle(text);
        }
    }
    item
}

fn push_arrival_extras(children: &mut Vec<Component>, data: &GuestData) {
    let video = data.config.arrival.arrival_video_url.trim();
    if is_https(video) {
        children.push(Component::Link(
            Link::new()
                .label("i18n:guest.watchVideo")
                .href(video.to_string())
                .action(external_action(video)),
        ));
    }

    let mut rank = 1;
    for step in &data.texts.steps {
        let title = step.title.trim();
        if title.is_empty() {
            continue;
        }
        children.push(Component::ListItem(arrival_step(step, rank)));
        rank += 1;
    }
}

pub fn build_access_glance(data: &GuestData) -> Vec<Component> {
    let mut children = Vec::new();

    push_reveal_banner(&mut children, data);

    if let Some(map) = property_map(data) {
        children.push(map);
    }

    // Les codes d'abord, en tuiles, et l'heure d'arrivée à côté : c'est ce que le voyageur ouvre
    // la carte pour trouver (§2.1).
    let mut tiles = secret_tiles(data);
    tiles.extend(arrival_tile(data));
    if !tiles.is_empty() {
        children.push(Component::Grid(
            Grid::new()
                .minColumnWidth(130.0)
                .plain(true)
                .children(tiles),
        ));
    }

    // L'adresse reste : c'est la ligne qu'on lit à un chauffeur, et elle ne tient pas dans un
    // sous-titre déjà pris par le moyen d'accès.
    if !data.address.is_empty() {
        children.push(kv_row("i18n:guest.address", &data.address, false));
    }

    // Où se trouve ce par quoi on entre : la boîte à clés, le lieu du rendez-vous, la banque
    // d'accueil. Une ligne, pas le bloc entier.
    push_method_location(&mut children, data);

    // Le nom du moyen, l'accès à l'immeuble et le parking ne sortent plus en rangées ici : la
    // maquette garde la carte à l'essentiel — le plan, les codes, le chemin — et le sous-titre
    // nomme déjà le moyen. Huit rangées entre les tuiles et le chemin poussaient les étapes
    // d'arrivée sous le pli, alors que ce sont elles qu'on relit une main sur la valise. Tout
    // reste dans la sous-page, à un doigt.

    // Le chemin jusqu'à la porte tient sur la carte, pas seulement dans la sous-page : c'est ce
    // qu'on relit en arrivant, une main sur la valise, sans vouloir ouvrir quoi que ce soit.
    push_arrival_path(&mut children, data);

    if let Some(url) = maps_url(data) {
        children.push(Component::Button(
            Button::new()
                .label("i18n:guest.openMaps")
                .variant(ButtonVariant::Outline)
                .action(external_action(&url)),
        ));
    }

    children
}

/// L'endroit du moyen d'accès, pour la carte d'accueil : une seule ligne, celle qui dit où aller.
///
/// Les moyens qui n'ont pas d'endroit à donner — un code de portail, une serrure connectée — n'en
/// poussent aucune : le code est déjà en tuile, et le clavier se trouve dans les étapes.
fn push_method_location(children: &mut Vec<Component>, data: &GuestData) {
    match &data.config.method {
        MethodFields::Keybox { location, .. } => {
            push_text_row(children, "i18n:guest.keybox.location", location)
        }
        MethodFields::InPerson { meeting_place, .. } => {
            push_text_row(children, "i18n:guest.inPerson.meetingPlace", meeting_place)
        }
        MethodFields::BuildingStaff { desk_location, .. } => push_text_row(
            children,
            "i18n:guest.buildingStaff.deskLocation",
            desk_location,
        ),
        _ => {}
    }
}

/// Les étapes d'arrivée, précédées de leur intertitre. Rien du tout quand l'hôte n'en a saisi
/// aucune — un intertitre seul annoncerait un chemin qui n'existe pas.
fn push_arrival_path(children: &mut Vec<Component>, data: &GuestData) {
    let mut steps = Vec::new();
    let mut rank = 1;
    for step in &data.texts.steps {
        if step.title.trim().is_empty() {
            continue;
        }
        steps.push(Component::ListItem(arrival_step(step, rank)));
        rank += 1;
    }
    if steps.is_empty() {
        return;
    }
    children.push(Component::Eyebrow(
        Eyebrow::new().text("i18n:guest.arrivalPath"),
    ));
    children.extend(steps);
}

pub fn build_access_detail(data: &GuestData) -> Vec<Component> {
    let mut children = Vec::new();

    let note = data.texts.global_note.trim();
    if !note.is_empty() {
        children.push(Component::InfoBanner(
            InfoBanner::new()
                .title("i18n:guest.note.title")
                .message(note.to_string()),
        ));
    }

    push_reveal_banner(&mut children, data);

    if let Some(map) = property_map(data) {
        children.push(map);
    }

    if !data.address.is_empty() {
        children.push(kv_row("i18n:guest.address", &data.address, false));
    }

    push_primary_method(&mut children, data, true);

    if let Some(building) = data.config.building_access.as_ref() {
        push_building_access(&mut children, data, building, true);
    } else if data
        .texts
        .building_note
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        push_building_access(&mut children, data, &BuildingAccess::default(), true);
    }
    push_parking(&mut children, data, data.config.parking.as_ref(), true);

    if let Some(url) = maps_url(data) {
        children.push(Component::Button(
            Button::new()
                .label("i18n:guest.openMaps")
                .variant(ButtonVariant::Outline)
                .action(external_action(&url)),
        ));
    }

    push_arrival_extras(&mut children, data);

    children
}

/// Le sous-titre de la carte : le moyen d'accès, et s'il se fait sans personne.
///
/// « Boîte à clés · entrée autonome » dit en une ligne ce que le voyageur veut savoir avant
/// d'ouvrir : par quoi il entre, et s'il doit attendre quelqu'un.
///
/// Les deux morceaux sont traduits ici, pas assemblés en `i18n:` : le shell résout une chaîne
/// entière, jamais un fragment, et `"i18n:a · i18n:b"` s'afficherait tel quel.
fn method_key(method: PrimaryMethod) -> &'static str {
    // Le fil est en snake_case, les clés en camelCase : les dériver l'une de l'autre donnerait
    // trois clés fausses sur sept, et le voyageur lirait « guest.method.door_code ».
    match method {
        PrimaryMethod::Keybox => "guest.method.keybox",
        PrimaryMethod::DoorCode => "guest.method.doorCode",
        PrimaryMethod::SmartLock => "guest.method.smartLock",
        PrimaryMethod::InPerson => "guest.method.inPerson",
        PrimaryMethod::BuildingStaff => "guest.method.buildingStaff",
        PrimaryMethod::HostGreets => "guest.method.hostGreets",
        PrimaryMethod::Other => "guest.method.other",
    }
}

pub fn access_summary(data: &GuestData) -> String {
    let key = method_key(data.config.primary_method);
    let method = translate(key, &Vars::new()).unwrap_or_else(|_| key.to_string());
    let autonomous = matches!(
        data.config.primary_method,
        PrimaryMethod::Keybox | PrimaryMethod::DoorCode | PrimaryMethod::SmartLock
    );
    let qualifier_key = if autonomous {
        "guest.entry.selfCheckin"
    } else {
        "guest.entry.greeted"
    };
    let qualifier =
        translate(qualifier_key, &Vars::new()).unwrap_or_else(|_| qualifier_key.to_string());
    format!("{method} · {qualifier}")
}
