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

## Verify the artifact

Every published version is signed by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify ghcr.io/portakiapp/portaki-modules-waste-recycling@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
