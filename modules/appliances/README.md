# appliances

Official Portaki appliance guide — structured device how-tos for guest booklets and the host dashboard.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`appliances`

OCI image: `ghcr.io/portakiapp/portaki-modules-appliances:<semver>`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | `AppliancesContent` entity |

## Content model

```json
{
  "safety_notice": "…",
  "devices": [
    {
      "id": "tv",
      "icon": "📺",
      "title": "Television",
      "subtitle": "Living room",
      "steps": ["Power on with the black remote."],
      "tip": "Remote on the TV stand.",
      "manualUrl": ""
    }
  ]
}
```

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Device list glance |
| guest | `explore.detail` | Full list (page) |
| guest | `explore.item` | Device how-to detail |
| host | `main` | Structured device slots + safety notices |

Host workspace tab: `pathSegment = "appliances"`.

Host edits fields; storage still keeps `content_fr` / `content_en` as structured JSON internally.

## Queries and commands

- `getContent`
- `saveContent`

## Known gap

`explore.item` openOverlay passes `deviceId` in args, but surface renderers currently receive only `Context` (no input params). Until the guest shell forwards overlay args, the item surface shows the first device.

## Development

```bash
cargo test -p appliances
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
