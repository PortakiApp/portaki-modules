# ev-parking

Official Portaki guest EV parking module — reserved spot, gate code, charger PIN, and timed reveal for guest booklets.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`ev-parking`

OCI image: `oci.portaki.app/modules/ev-parking:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (`spot_label`, `charger_pin`, `parking_code`, `map_url`, `instructions`, `reveal_policy`) |

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Spot label + masked or revealed codes |
| guest | `explore.detail` | Full EV parking block |
| host | `main` | Spot, codes, map link, instructions, reveal policy |

## Queries and commands

- `getConfig` / `updateConfig` — host KV config
- `emailContext` — `evParkingSpot` for Portaki `arrival` / `arrival-day` guest emails when `spot_label` is set

## Development

```bash
cargo test -p ev-parking
cd modules/ev-parking
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/ev-parking@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
