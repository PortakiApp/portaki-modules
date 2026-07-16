# emergency-contacts

Official Portaki emergency contacts module — useful numbers and host line for guest booklets.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`emergency-contacts`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (`contacts`, `host_visible_phone`) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Phone rows |
| guest | `explore.detail` | Rows + 112 banner (bottom sheet) |
| host | `main` | Host phone + contact slots |

## Development

```bash
cargo test -p emergency-contacts
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
