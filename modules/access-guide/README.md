# access-guide

Official Portaki access module — primary entry method, optional layers (building / parking / arrival), timed secret reveal, and a smart-lock provider hook.

## Module id

`access-guide`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | Legacy KV `config` + `texts/{lang}`, imported once by the platform |
| `access.smart_lock` | No (providers) | Declared by future lock modules (Nuki, Igloohome, Yale…) |

## Config model

Declared with `#[portaki_sdk::config]` (`HostConfig`): the platform stores it, validates the
host form (`updateConfig` never reaches the module) and blocks publication while
`primary_method` is empty. The keys are the host form names, flat:

```text
primary_method (required) + the fields of each method (keybox_*, door_code*, smart_lock_*, in_person_*, building_staff_*, host_greets_*)
building_access_enabled, building_access_gate_code, building_access_intercom
parking_enabled, parking_map_url, parking_code
address, arrival_lat, arrival_lng (numbers), arrival_video_url, reveal_policy
steps [{ id, kind, title, detail }]
method_instructions, building_note, parking_info, global_note
```

Codes (`keybox_code`, `door_code`, `smart_lock_manual_code`, `building_access_gate_code`,
`parking_code`) are secrets: never sent back to the form, blank keeps them. Every text a guest
reads (the notes, the steps, `keybox_location`, `in_person_meeting_place`…) is an `I18nText`: the
host edits the one of its dashboard language, the platform keeps the others; guests read their
language, else `fr`, `en`, any. The steps keep their stored order and send their `id`.
Guest surfaces, emails and `publishReadiness` (code required for keybox / door code / smart
lock without provider) read the nested `ModuleConfig` built from it.

### `primary_method`

`keybox` · `door_code` · `smart_lock` · `in_person` · `building_staff` · `host_greets` · `other`

### `reveal_policy`

| Preset | When secrets become visible |
|--------|-----------------------------|
| `always` | Immediately |
| `hours_before_24` | `checkinAt − 24h` |
| `day_before_16h` | Day before at 16:00 in property timezone (default) |
| `at_checkin` | From `checkinAt` |

Reveal logic lives **in this module**. Stay timing comes from generic SDK host fields (`checkinAt`, `checkoutAt`, `propertyTimezone`) — the platform does not encode access-guide rules.

### Email context (`emailContext`)

Stay-scoped query for Portaki guest emails (`arrival`, `arrival-day`, `new-code`).
`stay-link` uses the stay page token only — this module returns empty fields for that template.

Args (camelCase):

```json
{ "templateKey": "arrival", "checkinTimeFormatted": "16:00", "locale": "fr" }
```

Response (camelCase):

```json
{
  "arrivalCallout": "…",
  "entryAccessCode": "4821",
  "accessCodeLabel": "Code boîte à clés",
  "secretsRevealed": true,
  "revealAvailableFrom": null
}
```

Applies the same reveal policy as guest SDUI — plaintext codes are omitted while locked.

### Smart-lock hook

When `primary_method = smart_lock` and `smart_lock_provider_module_id` is set, guest SDUI emits `Action::command` (`unlock` / `getGuestCredential`) toward that module. Otherwise guests see `manual_code` / instructions only.

Host UI lists installed property modules that declare `capabilities.provided: ["access.smart_lock"]` via generic SDK host op `module.listByCapability`, plus **Manuel / autre**.

### Smart-lock provider contract

Future lock modules (`nuki`, `igloohome`, `yale`, …) must:

1. Declare `#[capability(provided, id = "access.smart_lock")]` (or `capabilities.provided` in the SDK manifest).
2. Expose guest commands `unlock` and `getGuestCredential` (stay/session args).
3. Use connectors / credential bindings for vendor APIs — never store OAuth tokens in module KV.

### Legacy migration

Before the platform held it, the module kept a nested `config` blob (or the older flat one:
`gate_code`, `keybox_code`, `steps_json`…) and its copy in `texts/{lang}`.
`#[portaki_sdk::config(legacy = legacy)]` maps all of it onto the declared keys (through
`migrate_legacy`, `texts/fr` / `texts/en` or the copy the blob embeds): the platform imports that
once (`legacyConfig`), and `HostConfig::load` reads it until then.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Method + secrets (masked or revealed) + parking / Maps |
| guest | `explore.detail` | Full guide + smart-lock CTAs |
| host | `main` | Conditional form: method, layers, reveal, provider |

## Development

```bash
cargo test -p access-guide
```

i18n: `i18n/fr-FR.json`, `i18n/en-US.json` — mirror into the dashboard with `pnpm generate:module-host-i18n`.

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
