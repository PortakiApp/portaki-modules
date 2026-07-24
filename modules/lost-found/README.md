# lost-found

Official Portaki lost & found — host-declared found items (guest email) and guest self-reports (host email).

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`lost-found`

OCI image: `ghcr.io/portakiapp/portaki-modules-lost-found:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `LostFoundReport` entity (many per stay) + KV config (`host_note`) |

## Data model

`LostFoundReport` (schema v2):

| Field | Notes |
|-------|--------|
| `kind` | `lost` \| `found` |
| `item_description` | Plain text (guest) or TipTap JSON (host-found) |
| `status` | `to_collect` (default, « À récupérer ») \| `sent` (« Envoyé ») \| `returned` (« Récupéré ») |
| `contact_hint` / `details` | Guest optional fields |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Kind + description form; optional host tip banner; stay report list after submit |
| host | `main` | Design editor: info banner, TipTap guest note (`host_note`), recent reports with status pills + update |
| host | `stay` | Design declare modal: header (`guestName` · `stayDates`), description TextArea, send CTA; stay report list + status |

Host apps only embed `HostSurfacePanel` (or equivalent). No module-named React create modal.
Create always defaults status to `to_collect` — no status field on create.

## Queries and commands

- `listForStay` — guest stay reports; host may pass `stayId`
- `listRecent` — newest reports for the property (host)
- `submit` — guest create report; `host::email::send` → host notify (module SDUI)
- `submitFound` — host create found report(s); `host::email::send` → guest (module SDUI)
- `sendCheckoutFollowUp` — J+2 tick; guest mail only when a stay declaration exists
- `updateStatus` — host change report status (`to_collect` \| `sent` \| `returned`) after create
- `updateConfig` — persists optional `host_note` in KV (TipTap JSON ok)
- `emailContext` — optional Portaki snippets: `checkoutTips`, `lostItemDescription` + `hasDeclaration`

## Development

```bash
cargo test -p lost-found
cd modules/lost-found
portaki build --release
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
