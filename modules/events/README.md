# events

Official Portaki events module — OpenAgenda nearby happenings plus optional host-curated slots.

## Module id

`events`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config + nearby cache |
| `external.open-agenda.pool` | Optional | Platform OpenAgenda API key |
| `external.open-agenda.byok` | Optional | Workspace OpenAgenda API key |

## Connectors

| Id | Auth | Operations |
|----|------|------------|
| `open-agenda` | `query_key` (`?key=`) | `nearby_events` → `GET /v2/events` |

Pool env on module-runtime: `OPENAGENDA_POOL_KEY`.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Upcoming events (manual + nearby) |
| guest | `explore.detail` | Full list, map when coordinates exist |
| host | `main` | Public events toggle / radius, events table (side panel per row, sorted by date), disclaimer |
| host | `calendar` | Workspace tab: month calendar (`MonthCalendar`) of your events (recurrences expanded) and OpenAgenda's, source filter, duplicates banner, event detail |

## Behaviour

- Property `lat` / `lng` from host context; radius from module config (default 15 km, 1 to 50).
- Nearby results cached in KV (`nearby_cache`, ~1h); refresh on render miss or `refreshNearby`.
- Manual slots win over an OpenAgenda duplicate: same title, starts within 1 h, venues within 500 m (`nearby::is_duplicate`, shared by the booklet and the calendar tab).
- One OpenAgenda call fetches the next 100 upcoming events; the calendar tab only sees those (no past months).

## Development

```bash
cargo test -p events
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/events@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
