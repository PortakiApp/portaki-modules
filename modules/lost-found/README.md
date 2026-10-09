# lost-found

Official Portaki lost & found — host-declared found items (guest email) and guest self-reports (host email).

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`lost-found`

OCI image: `oci.portaki.app/modules/lost-found:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `LostFoundReport` entity (many per stay) |

## Data model

`LostFoundReport` (schema v2):

| Field | Notes |
|-------|--------|
| `kind` | `lost` \| `found` |
| `item_description` | Plain text (guest) or TipTap JSON (host-found) |
| `status` | `declared` (« Déclaré », guest report) \| `found` (« Trouvé », host-declared) \| `shipped` (« Renvoyé ») \| `picked_up` (« Retiré ») \| `donated` (« Donné ») \| `not_found` (« Introuvable »). Legacy `to_collect` / `sent` / `returned` rows read as `declared`-or-`found` / `shipped` / `picked_up` |
| `contact_hint` / `details` | Guest optional fields |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Kind + description form; stay report list after submit |
| host | `lost-stats` | `property-stats-detail` — counters (declared, returned, waiting, no answer) and the declared items as `FeedItem`s, no chart; a row opens the dashboard modal (`host.surface.overlay`), which renders this surface with `input.itemId` as the item detail |
| host | `items` | `property-workspace-tab` — every item, its status, the guest's return choice and shipping address, and a status `Select` limited to the allowed transitions |
| host | `create` | Stay-action modal body: TipTap description (`RichTextEditor`), hint, « Envoyer au voyageur » |
| host | `stay` | Stay-detail Card list + status when reports exist; empty tree when none (no empty-state copy) |

Host apps only embed `HostSurfacePanel` (or equivalent). No module-named React create modal.
Stay-action « Déclarer un objet trouvé » stays available even when the stay list is empty.
A guest report starts `declared`, a host-declared item `found` — no status field on create.

## Queries and commands

- `listForStay` — guest stay reports; host may pass `stayId`
- `listRecent` — newest reports for the property (host)
- `submit` — guest create report; `host::email::send` → host notify (module SDUI)
- `submitFound` — host create found report(s); `host::email::send` → guest (module SDUI)
- `sendCheckoutFollowUp` — J+2 tick; guest mail only when a stay declaration exists
- `updateStatus` — host moves a report along the allowed transitions (`declared` → any; `found` → `shipped` \| `picked_up` \| `donated`; `not_found` → `found`; returns are final); records `workspace-activity.record`
- `timelineTasks` / `taskToggle` / `taskComplete` — « Objet à renvoyer » in À venir for each item the guest asked to have shipped, until it is `shipped`; completing the task ships it
- `statsSummary` — tile `lost-stats`: items declared over the period, waiting ones as attention
- `emailContext` — optional Portaki snippets: `checkoutTips`, `lostItemDescription` + `hasDeclaration`

## Development

```bash
cargo test -p lost-found
cd modules/lost-found
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/lost-found@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
