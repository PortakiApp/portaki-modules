# sections

Official Portaki welcome module (« Bienvenue ») — the host's word of welcome and the booklet's long texts. A **core** module (`portaki_module!(core)`): installed on every property, always active, no switch, not in the modules list.

Part of the [`portaki-modules`## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | Reads the sections of before the config (`SectionItem` + `SectionItemLocale`) |

## Configuration (`#[portaki_sdk::config]`, held by the platform)

- `show_from` — `booking` · `day_before` (default) · `arrival_day`: when the guest sees the word.
- `sections` — 0 to 10 rows `{ id, title: I18nText, body: I18nText (TipTap JSON), photo }`; the first
  is the word of welcome. `None` until the first save: the legacy tables are read instead.
- Copy from another listing: titles, texts and `show_from`; photos are never copied.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Teaser + first sections (absent before `show_from`) |
| guest | `explore.sheet` | Full bodies, each opened on its photo (bottom sheet) |
| host | `main` | `property-listing-card`: the « Message d'accueil » card of « Fiche & coordonnées » |

## Queries

- `listSections`
- `publishReadiness`

 |

Host workspace tab: `pathSegment = "sections"`.

## Queries and commands

- `listSections`
- `saveSection`
- `deleteSection`
- `reorder`

## Development

```bash
cargo test -p sections
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/sections@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
