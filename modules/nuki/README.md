# nuki

Official Portaki smart-lock **provider** for the [`access.smart_lock`](https://github.com/PortakiApp/portaki-sdk) contract. Guest unlock UX lives in **access-guide** — this module has no guest surfaces.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`nuki`

## Binding access-guide

1. Install this module on the property.
2. Open **Accès & parking** (access-guide) → primary method **Smart lock**.
3. Set **Provider** to **nuki** (`smart_lock_provider_module_id = "nuki"`).
4. Configure keypad code here (Modules → Nuki).
5. Optional: save a Nuki Web API token under workspace **Integrations** (provider `nuki`) for remote unlock.

When codes are revealed, access-guide sends guest commands `unlock` and `getGuestCredential` to this module with optional `{ "stayId": "…" }`.

## Capabilities

| Capability | Role |
|------------|------|
| `access.smart_lock` | **Provided** — peer discovery for access-guide |
| `core.storage` | **Required** — the pre-platform KV config, read once by the platform import (`legacyConfig`) |
| `external.nuki.byok` | **Optional** — Nuki Web API token for `POST …/action/unlock` |

## Config

Held by the platform (`#[portaki_sdk::config]`), which takes `updateConfig`; `keypad_code` is a
`secret`, never sent back to the host form. `publishReadiness` blocks publication until there is a
keypad code, or remote unlock (`external.nuki.byok` granted + `smartlock_id`).

```json
{
  "smartlock_id": "…",
  "keypad_code": "……",
  "device_name": "…"
}
```

## Guest commands

| Command | Behavior |
|---------|----------|
| `getGuestCredential` | `{ "type": "keypad", "code": "…", "smartlockId": "…" }` — errors if `keypad_code` empty |
| `unlock` | Prefer remote unlock when BYOK + `smartlock_id` are set (`mode: "remote"`). On failure or missing token, fall back to keypad (`mode: "credential_fallback"`). |

Remote unlock uses the module connector `nuki` / `remote_unlock` (Bearer, path `/smartlock/{smartlockId}/action/unlock`). Requires module-runtime egress that supports POST + Bearer (shipped in portaki-platform runtime).

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| host | `main` | Smart lock ID, keypad code, device name |

## Development

```bash
cargo test -p nuki
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
