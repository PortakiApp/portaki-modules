# sections

Official Portaki editorial sections — title + markdown body blocks for the guest booklet.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`sections`

OCI image: `oci.portaki.app/modules/sections:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `SectionItem` + `SectionItemLocale` |

## Data model

- `section_item` — id, sort_order
- `section_item_locale` — section_id, lang, title, body_markdown

No TipTap — markdown strings only.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Teaser + first sections |
| guest | `explore.sheet` | Full bodies (bottom sheet) |
| host | `main` | Form editor for primary section |

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
