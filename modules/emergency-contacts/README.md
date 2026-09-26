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

## Verify the artifact

Every published version is signed by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify ghcr.io/portakiapp/portaki-modules-emergency-contacts@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
