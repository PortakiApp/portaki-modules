# rules

Official Portaki house rules module — structured bilingual items for guest booklets and the host dashboard.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`rules`

OCI image: `ghcr.io/portakiapp/portaki-modules-rules:<semver>`

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
| guest | `home.card` | Séjour glance (up to 4 icon rows) → fullscreen |
| guest | `explore.detail` | Full rules list in elevated card (page body) |
| host | `main` | « Règles du logement » StepList (`rules-editor-v1`) |

Host workspace tab: `pathSegment = "rules"` (see `portaki.module.json`).

Host edits fields (icon, titles, subtitles). Storage still keeps `content_fr` / `content_en` as structured JSON internally.

## Queries and commands

- `getContent` — locale-aware items + raw FR/EN JSON
- `updateConfig` — workspace Save chrome → upsert `RulesContent` (`items[]`)
- `saveContent` — same payload (legacy / direct command)

## Development

```bash
cargo test -p rules
cd modules/rules
portaki build --release
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
