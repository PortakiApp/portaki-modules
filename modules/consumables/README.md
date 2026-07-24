# consumables

Official Portaki consumables & stock — host catalog per property, guest shortage reports during the stay, restock tracking.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`consumables`

OCI image: `ghcr.io/portakiapp/portaki-modules-consumables:<semver>`

## Install / pin

Install from the Portaki module catalogue (or pin the OCI digest for the property). Semver tag:

`ghcr.io/portakiapp/portaki-modules-consumables:0.1.0`

Properties pin a digest — bump semver on every runtime change before republish.

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `ConsumableItem` (property catalog) + `ConsumableReport` (stay shortages) |

## Data model

`ConsumableItem` (schema v1): bilingual labels (`label_fr` JSON map + legacy `label_en`), `sort_order`, optional `low_threshold` (0 = unused in v0.1 UI).

`ConsumableReport` (schema v1): `stay_id`, `item_id`, `item_label` snapshot, `level` (`missing` \| `low`), optional `note`, `status` (`open` \| `restocked`).

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Catalog ChoiceList + missing/low + note; stay report list |
| host | `main` | Catalog editor (Save → `updateConfig`), seed defaults, open reports + mark restocked |
| host | `stay` | Stay-detail reports when any exist (empty tree otherwise) |
| host | `stock` | Stats card: catalog size + open reports |

Host apps only embed `HostSurfacePanel` / stats registry. No module-named React feature.

## Queries and commands

- `listItems` — property catalog
- `listForStay` / `listRecent` — shortage reports
- `listOpenCount` — open report count (stats)
- `replaceItems` / `updateConfig` — host catalog replace
- `seedDefaults` — fill empty catalog with common consumables (FR/EN)
- `submit` — guest report; `host::email::send` → host notify
- `updateStatus` — host `open` \| `restocked`

## UX notes (v0.1)

No design screen for stock existed — flow mirrors checklist (catalog) + lost-found / issue-report (guest signal + host status). Guest reports one catalog item per submit (can send several). Host restock is a one-click status flip, not a quantity ledger yet (`low_threshold` reserved).

## Development

```bash
cargo test -p consumables
cd modules/consumables
portaki build --release
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
