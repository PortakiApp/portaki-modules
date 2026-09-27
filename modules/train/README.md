# train

Official Portaki train module — nearest station schedule glance for guest booklets.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`train`

OCI image: `oci.portaki.app/modules/train:<semver>`

## Capabilities

None. v0.1 has no host editor and no storage — station info and destination
schedules are static Rust constants in `src/content.rs`. A future pass will
read this from module config and/or a Navitia connector.

## Content model

Static, hardcoded in `src/content.rs`:

- Nearest station label + distance (`DEFAULT_STATION_LABEL`, `default_station_distance`)
- Destinations: `Nice-Ville`, `Cannes`, `Monaco`, `Grasse`
- Mock TER SUD PACA departure times per destination (`schedule_for`)

## Surfaces

| Shell | Surface id | Description |
|-------|------------|--------------|
| guest | `home.card` | Mixed-destination departure board glance (4 rows) |
| guest | `explore.detail` | From/to header, destination filter chips, next departures |

Guest route: `pathSegment = "train"` (see the guest `#[surface]`s).

Destination filter chips re-navigate to `train` with `{ "dest": "<destination>" }`
params, read back via `ctx.input.dest` in `render_explore_detail`.

## Development

```bash
cargo test -p train
cd modules/train
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/train@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
