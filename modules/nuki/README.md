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
| `external.nuki.byok` | **Optional** — Nuki Web API token for remote unlock and per-stay codes |

## Config

Held by the platform (`#[portaki_sdk::config]`), which takes `updateConfig`; `keypad_code` is a
`secret`, never sent back to the host form. `publishReadiness` blocks publication until there is a
keypad code, or remote unlock (`external.nuki.byok` granted + `smartlock_id`).

```json
{
  "smartlock_id": "…",
  "keypad_code": "……",
  "device_name": "…",
  "code_per_stay": true
}
```

### One code per stay (`code_per_stay`, default `true`)

With the Nuki Web key and a lock ID, the guest's first reveal looks for a keypad authorization
named `Portaki <first 8 chars of the stay id>` on the lock (`GET /smartlock/{id}/auth`) and, if
none, creates one (`PUT /smartlock/{id}/auth`, type 13) valid from check-in to check-out. Nuki is
the store: no KV (host-side writes would land in the draft layer the guest never reads). The
catalogue opens no delete or update, so codes are only bounded by `allowedUntilDate`. Any Nuki
error, or a lock that did not answer `unlock`, falls back to `keypad_code` (spec §9 #1).

## Guest commands

| Command | Behavior |
|---------|----------|
| `getGuestCredential` | `{ "type": "keypad", "code": "…", "smartlockId": "…" }` — errors if `keypad_code` empty |
| `unlock` | Prefer remote unlock when BYOK + `smartlock_id` are set (`mode: "remote"`). On failure or missing token, fall back to keypad (`mode: "credential_fallback"`). |

Remote unlock uses the module connector `nuki` / `remote_unlock` (Bearer, path `/smartlock/{smartlockId}/action/unlock`). Requires module-runtime egress that supports POST + Bearer (shipped in portaki-platform runtime).

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| host | `main` | Smart lock ID, keypad code, device name, one code per stay |
| host | `stay` | Stay detail: access window and the stay's code, read from the lock |

## Development

```bash
cargo test -p nuki
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/nuki@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
