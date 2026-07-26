# guest-reviews

Official Portaki guest reviews module — post-stay thank-you, Airbnb CTA + QR, and/or Portaki star form.

## Module id

`guest-reviews`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config + submitted Portaki reviews |

## Config

Multi-select platforms (host toggles):

| Field | Meaning |
|-------|---------|
| `platform_airbnb` | Offer Airbnb review link (+ optional QR) |
| `platform_portaki` | Offer in-booklet Portaki star form |
| `airbnb_review_url` | Required when Airbnb is selected |
| `show_qr_code` | QR under the Airbnb CTA |
| `thank_you_message` | Localized thank-you copy |

Legacy `review_channel` (`airbnb` / `portaki` / `both`) is still read and mapped to the toggles.

Guest CTAs only appear for **selected and feasible** platforms — Airbnb without a URL is never shown as a dead button.

## Events

`submitReview` emits `guest-reviews.submitted` (`propertyId`, `rating`, `comment`, optional
`guestName`). Module sends host transactional email via `host::email::send`.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Inline thank-you + review CTAs (no overlay) |
| host | `main` | Platform toggles, Airbnb URL, QR toggle, thank-you message |

## Commands

- `updateConfig` — persist host settings (validates ≥1 platform; Airbnb requires URL)
- `submitReview` — store Portaki rating + comment in KV (requires Portaki enabled)

## Development

```bash
cargo test -p guest-reviews
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
