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
| host | `main` | Nearby toggle / radius + six manual slots + disclaimer |

## Behaviour

- Property `lat` / `lng` from host context; radius from module config (default 40 km).
- Nearby results cached in KV (`nearby_cache`, ~1h); refresh on render miss or `refreshNearby`.
- Manual slots win on title+start collisions.

## Development

```bash
cargo test -p events
```

## Verify the artifact

Every published version is signed by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify ghcr.io/portakiapp/portaki-modules-events@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
