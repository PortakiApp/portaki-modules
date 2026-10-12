# rules

Official Portaki house rules module — structured bilingual items for guest booklets and the host dashboard.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`rules`

OCI image: `oci.portaki.app/modules/rules:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `RulesContent` entity |

## Content model

`content_fr` / `content_en` store structured JSON (not TipTap):

```json
{
  "items": [
    { "icon": "clock-circle", "title": "Quiet after 10 pm", "subtitle": "Please respect neighbours" }
  ]
}
```

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Séjour glance (the first `card_limit` rules, 2–6, default 4) → fullscreen |
| guest | `explore.detail` | Full rules list in elevated card (page body) |
| host | `main` | Page: « Affichage » (`card_limit`) + « Règles », a sortable `DataTable` whose rows open in a dialog (`rules-editor-v1`) |

Host workspace tab: `pathSegment = "rules"` (the `main` surface's `#[surface]`).

Each rule has a stable `id` shared by every language (legacy rules get `r{index}` on read); the host order is the booklet order in every language. Host edits title, detail (`subtitle` in storage), type, theme and hours (`HH:MM-HH:MM`). Storage still keeps `content_fr` / `content_en` as structured JSON internally.

## Queries and commands

- `getContent` — locale-aware items + raw FR/EN JSON
- `updateConfig` — autosave → upsert `RulesContent` (`rules[]` by id, in display order; legacy `items[]` by index)
- `applyTemplate` — « Partir d'un modèle » on an empty rulebook (silence, pets, smoking, parties, visitors)
- `saveContent` — same payload (legacy / direct command)

## Development

```bash
cargo test -p rules
cd modules/rules
portaki build --release
```

## Verify the artifact

Every published version is attested by this repository's `ci` workflow on `main` (keyless cosign,
Rekor transparency log), with SLSA provenance and the `cargo audit` report attached. A Portaki
production runtime only runs a signed digest. To check one yourself:

```bash
cosign verify-attestation --type slsaprovenance1 oci.portaki.app/modules/rules@<digest> \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  --certificate-identity https://github.com/PortakiApp/portaki-modules/.github/workflows/ci.yml@refs/heads/main
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
