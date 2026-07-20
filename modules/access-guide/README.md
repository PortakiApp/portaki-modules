# access-guide

Official Portaki access module — primary entry method, optional layers (building / parking / arrival), timed secret reveal, and a smart-lock provider hook.

## Module id

`access-guide`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (method, layers, reveal, provider binding) |
| `access.smart_lock` | No (providers) | Declared by future lock modules (Nuki, Igloohome, Yale…) |

## Config model

```text
primary_method + method fields
optional: building_access | parking | arrival
reveal_policy (default: day_before_16h)
smart_lock_provider_module_id?   # when primary_method = smart_lock
```

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

### Smart-lock hook

When `primary_method = smart_lock` and `smart_lock_provider_module_id` is set, guest SDUI emits `Action::command` (`unlock` / `getGuestCredential`) toward that module. Otherwise guests see `manual_code` / instructions only.

Host UI currently accepts a manual provider module id (installed `access.smart_lock` listing is still a stub).

### Legacy migration

On KV load, flat `gate_code` / `keybox_code` (+ parking / steps / address) are migrated via `migrate_legacy`:

- non-empty `keybox_code` → `primary_method = keybox`
- else non-empty `gate_code` → `primary_method = door_code`
- parking / steps / video / note / address → optional layers under `arrival` / `parking` / `building_access`
- default reveal → `day_before_16h`

Config is then re-serialized in the new shape.

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
