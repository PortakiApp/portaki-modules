# access-guide

Official Portaki access module — primary entry method, optional layers (building / parking / arrival), timed secret reveal, and a smart-lock provider hook.

## Module id

`access-guide`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV `config` + `texts/{lang}` (method, layers, reveal, provider binding) |
| `access.smart_lock` | No (providers) | Declared by future lock modules (Nuki, Igloohome, Yale…) |

## Config model

Two KV documents — structural vs per-language copy:

```text
config                  # language-invariant structure
texts/{lang}            # fr | en | … from ctx.locale (fr-FR → fr)
```

### Shared `config`

```text
primary_method + method fields (codes, locations, GPS — no free-text instructions)
optional: building_access { gate_code?, intercom? } | parking { map_url, code? }
arrival { address, steps[{ id, kind }], arrival_video_url }
reveal_policy (default: day_before_16h)
smart_lock_provider_module_id?   # when primary_method = smart_lock
```

### Per-lang `texts/{lang}`

```text
method_instructions?
building_note?
parking_info
global_note
steps: [{ id, title, detail? }]   # titles/details only; kinds stay on shared steps
```

Host `render_host` / `updateConfig` load and save texts for the **active** `ctx.locale` only.
Guest resolves texts: guest locale → property default → `fr` → first available KV key.

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

On KV load, flat `gate_code` / `keybox_code` (+ parking / steps / address) are migrated via `migrate_legacy`:

- non-empty `keybox_code` → `primary_method = keybox`
- else non-empty `gate_code` → `primary_method = door_code`
- parking / steps / video / address → optional layers under `arrival` / `parking` / `building_access`
- default reveal → `day_before_16h`
- embedded free-text (`method.instructions`, `building_access.note`, `parking.info`, `global_note`, step title/detail, including legacy `{ fr, en }` objects) → seeded into `texts/fr` (and `texts/en` when bilingual), then stripped from `config` so they cannot overwrite texts on later loads

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
