# access-guide

Official Portaki access guide module — arrival steps, codes, and parking.

## Module id

`access-guide`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (steps, codes, map/video URLs) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Address / codes / parking + Maps CTA |
| guest | `explore.detail` | Steps + links (page overlay) |
| host | `main` | Codes + structured step slots |

## Development

```bash
cargo test -p access-guide
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
