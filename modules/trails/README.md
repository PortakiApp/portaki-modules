# trails

Official Portaki trails module — the walks around the property, grouped by level, with duration,
distance and ascent.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`trails`

OCI image: `oci.portaki.app/modules/trails:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | Host configuration (`trails`, `commune_url`) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Level badges with counts + the first two walks |
| guest | `explore.detail` | The list, one section per level, plus the town link |
| guest | `explore.item` | One walk: level, trailhead, four measures, map, before you go |
| host | `main` | Up to twelve dynamic rows + the town link |

## Queries and commands

- `publishReadiness` — refuses to publish without one complete walk; recommends a level and
  measures on the rows that lack them
- `mapMarkers` — the trailheads, for the booklet's Map
- `updateConfig` — the platform persists the host form itself

## Design notes

- **Three fixed levels** (easy, moderate, demanding), translated, written as shape **and** text
  (▲ / ▲▲ / ▲▲▲). Never as a colour: a difficulty is not a status, and green/amber/red would read
  as one.
- A walk **without a level is not shown**. There is no default on purpose — calling a 600 m climb
  "easy" because the host left the field empty sends someone up it in sandals.
- A missing measure hides its tile rather than drawing an empty one.
- `starts_far` (≥ 1 km from the property) decides whether the page offers directions to the
  trailhead; under 120 m the walk starts at the door, and the map carries one marker, not two.
- **No GPX, no photo, no route line** in v1: `Map` has no `path` field, and `core.images` has no
  host API in the SDK. "Open the route" is the host's third-party link (Visorando, IGN), which is
  what §2.23 names for that button. The plate's "no trace, no photo" case is therefore this whole
  version.

## Development

```bash
cargo test -p trails
cd modules/trails
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/trails@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
