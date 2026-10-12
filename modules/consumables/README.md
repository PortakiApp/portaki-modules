# consumables

Official Portaki consumables & stock — host catalog per property, guest shortage reports during the stay, restock tracking.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`consumables`

OCI image: `oci.portaki.app/modules/consumables:<semver>`

## Install / pin

Install from the Portaki module catalogue (or pin the OCI digest for the property). Semver tag:

`oci.portaki.app/modules/consumables:0.1.0`

Properties pin a digest — bump semver on every runtime change before republish.

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `ConsumableItem` (property catalog) + `ConsumableReport` (stay shortages) |

## Data model

`ConsumableItem` (schema v1): bilingual labels (`label_fr` JSON map + legacy `label_en`), `sort_order`, optional `low_threshold` (0 = unused in v0.1 UI).

`ConsumableItem` (schema v5) adds `location` — « Où le trouver », a JSON map of every language like `label_fr`.

`ConsumableReport` (schema v5): `stay_id`, `item_id`, `item_label` snapshot, `level` (`missing` \| `low`), optional `note`, `status` (`open` \| `planned` \| `restocked`), `restocked_at`, `host_reply`, and the guest name and stay dates snapshotted at submit (`guest_name`, `stay_check_in`, `stay_check_out`).

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Catalog ChoiceList + missing/low + note; stay report list |
| host | `main` | Settings page (`property-workspace-tab`, page mode): the products table (`DataTable`, `rowName = "items"`, one side panel per row, rows keyed by id), the request settings, then the requests received (table + reply / Prévu / Livré) |
| host | `stay` | Stay-detail: « N demande(s) · M à traiter », the reports, « Voir les demandes » |
| host | `stock` | `property-stats-detail`: tracked, to buy, reported, last restock, reports per item and stock status (`FeedItem`, click opens the config). The `property-stats-card` tile of the same key is served by `statsSummary` |

Host apps only embed `HostSurfacePanel` / stats registry. No module-named React feature.

## Queries and commands

- `listItems` — property catalog
- `listForStay` / `listRecent` — shortage reports
- `listOpenCount` — open report count (stats)
- `replaceItems` / `updateConfig` — host catalog replace
- `seedDefaults` — fill empty catalog with common consumables (FR/EN)
- `submit` — guest report; `host::email::send` → host notify when « E-mail » is checked
- `updateStatus` — host `open` \| `planned` \| `restocked`; a new `hostReply` is emailed to the guest (`host-reply`, declared guest email)

## UX notes (v0.1)

No design screen for stock existed — flow mirrors checklist (catalog) + lost-found / issue-report (guest signal + host status). Guest reports one catalog item per submit (can send several). Host restock is a one-click status flip, not a quantity ledger yet (`low_threshold` reserved).

## Development

```bash
cargo test -p consumables
cd modules/consumables
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/consumables@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
