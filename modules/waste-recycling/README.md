# waste-recycling

Official Portaki waste & recycling module — bin-by-bin sorting rules and collection days for guest booklets.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`waste-recycling`

OCI image: `ghcr.io/portakiapp/portaki-modules-waste-recycling:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (`bins`, `collection_schedule`) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Bin rows + collection banner |
| guest | `explore.detail` | Enriched bins (bottom sheet) |
| host | `main` | Structured bin slots + schedule form |

## Queries and commands

- `getConfig` — reads KV configuration
- `updateConfig` — persists host settings in KV

## Development

```bash
cargo test -p waste-recycling
cd modules/waste-recycling
portaki build --release
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
