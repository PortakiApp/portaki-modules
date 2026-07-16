# local-guide

Official Portaki local guide module — nearby spots and host picks.

## Module id

`local-guide`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (`spots`, `disclaimer`) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Spot rows + tags |
| guest | `explore.detail` | Enriched spots (bottom sheet) |
| host | `main` | Spot slots + disclaimer form |

## Development

```bash
cargo test -p local-guide
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
