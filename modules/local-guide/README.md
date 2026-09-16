# local-guide

Official Portaki local guide module — nearby spots and host picks.

## Module id

`local-guide`

## Capabilities

| Capability | Required | Purpose |
|------------|----------|---------|
| `core.storage` | Yes | KV config (`spots`, `disclaimer`, `activities`) |

## Sections

| Section | Description |
|---------|-------------|
| Spots | Up to 6 host-curated addresses with category, distance, tag and note |
| Activities & tickets | GetYourGuide affiliate links (**off by default**) — an automatic destination search plus up to 10 host-curated links |

### Activities & tickets

Affiliate **links only**: no API call, no connector, no extra permission, and no
script, iframe or image loaded from GetYourGuide.

The section is **off by default**: `activities.enabled` defaults to `false`, and
a config with no `activities` key renders nothing — publishing this version turns
no existing install into an affiliate surface. These links earn Portaki a
commission, so the host opts in first, and the host sheet says so in every
locale, including that the host earns nothing. Once on, the section needs no
further configuration.

The destination is the host's `destination` override when set, otherwise the city
parsed from the property address (`PropertyContext::address` — there is no `city`
field). With neither, the section does not render.

Host-curated links are accepted only on `getyourguide.com` (any subdomain, any
path) or the `gyg.me` short domain; anything else is refused at save time with
`activities_url_not_getyourguide`. Long links get the partner id query parameter
set: added when absent, and **replaced when the host pasted one of their own** —
the commission is platform-wide, and the host sheet states plainly that a link
pasted here is rewritten with Portaki's affiliate id. Short links are left
untouched because they already carry an id.

A **partner disclosure is mandatory** and renders under the list in every locale
(`guest.activities.disclosure`) — the commission goes to Portaki, and the guest
reads that before clicking, not after.

The affiliate id lives in a single constant, `PARTNER_ID` in
[`src/affiliate.rs`](src/affiliate.rs). It is public by nature (it travels in
every link), so it is not a secret — but it is compiled into the Wasm, so
changing it means a module release. It is **empty by default**: while it is
empty, links render with no partner parameter at all.

## Surfaces

| Shell | Surface id | Description |
|-------|------------|-------------|
| guest | `home.card` | Spot rows + tags + activities section |
| guest | `explore.detail` | Enriched spots + activities section (bottom sheet) |
| guest | `upcoming.card` | Compact spot count |
| host | `main` | Spot slots + activities + disclaimer form |

## Development

```bash
cargo test -p local-guide
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
