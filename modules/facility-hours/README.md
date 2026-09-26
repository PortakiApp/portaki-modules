# facility-hours

Official Portaki facility hours module — pool, spa, and shared amenity schedules.

## Module id

`facility-hours`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (`facilities`, `general_note`) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | KeyValue hour rows |
| guest | `explore.detail` | Enriched list (page overlay) |
| host | `main` | Facility slots + note form |

## Development

```bash
cargo test -p facility-hours
```

## Verify the artifact

Every published version is signed by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify ghcr.io/portakiapp/portaki-modules-facility-hours@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
