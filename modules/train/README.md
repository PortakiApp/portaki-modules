# train

Official Portaki train module — nearest station schedule glance for guest booklets.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`train`

OCI image: `oci.portaki.app/modules/train:<semver>`

## Capabilities

The `sncf` connector, declared by the module itself (ADR-0021) against SNCF's Navitia API, with
**no host key**: one publisher key serves every property, as for Viator. Every operation declares
`sends = "module_config"` — the station the host wrote, and nothing else. No guest data and no
property address leave the module.

| Operation | Call | Cache |
|-----------|------|-------|
| `find_place` | `GET /coverage/sncf/places` | 24 h (gateway) + 30 days (KV), keyed by station name |
| `departures` | `GET /coverage/sncf/stop_areas/{id}/departures` | 2 min (KV) |
| `arrivals` | `GET /coverage/sncf/stop_areas/{id}/arrivals` | 2 min (KV) |

Needs the `kv` feature for those caches. Without the host's clock nothing is shown: judging
freshness on a wrong time would serve the same board forever.

## Content model

Nothing is hardcoded. The host gives **one** field — the station's name, as they call it
(`Gare d'Antibes`, `Antibes`) — plus an optional line shown under the board. The module resolves
the name to a Navitia `stop_area` itself and keeps the mapping.

The destinations offered to the guest are the distinct `display_informations.direction` values of
the board itself, in the order they appear there — the order of the next train, which beats the
alphabet. No station list is written in the module, and none is asked of the host.

### The night with no train

`/departures` is called without `from_datetime`: Navitia defaults to *now* and looks 24 h ahead.
A guest reading the booklet at 2 a.m. therefore gets the first morning train, marked `Demain`
when its local date differs from the property's — which is what distinguishes a night with no
train from an empty board.

### The response shape is documented, not captured

`tests/fixtures/sncf-*.json` are written from [doc.navitia.io](https://doc.navitia.io/), **not**
captured from a real call: Portaki has no SNCF key yet. Every field is therefore `Option`, and a
body nobody can read gives the error template rather than a wrong board. The first real call will
confirm or fix those fixtures, and `src/sncf.rs` is the only place to touch.

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
