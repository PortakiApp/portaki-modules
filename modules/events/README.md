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

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
