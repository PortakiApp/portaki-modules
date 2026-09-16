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
| Map | The located spots plus the property, on the enriched surface only |
| Activities & tickets | GetYourGuide affiliate links (**off by default**) — an automatic destination search plus up to 10 host-curated links |

### Map

Each spot carries an optional `address` / `lat` / `lng`, filled from the
`AddressMapPicker` in the host sheet. The map renders **only on
`explore.detail`**, and **only when at least one spot is located** — configs
written before the map carry no coordinates, and an empty map is worth less than
no map.

It is static and non-interactive, like the one in `events`: the surface is a
scrolling bottom sheet, where a pannable map would steal the guest's scroll
gesture.

Coordinates are filtered before they reach the map: outside WGS-84 bounds, and
the `0, 0` point — where no host has a good address, but where any form that
submitted empty fields lands. The property is added as a `property` marker and
counts in the centring, so the guest sees where they are starting from.

The three picker fields are submitted together. A caller older than the map sends
none of them, and the stored position is **kept** rather than wiped on the next
save.

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

The `destination` field accepts **either a place name or a GetYourGuide URL**:

- **Free text** (`Antibes`) builds the search link `…/s/?q=Antibes`. That search
  picks its results from the visitor's IP and ignores the query for a guest
  arriving from abroad or through a VPN — fine for most guests, unreliable for
  the rest.
- **A pasted URL** (`https://www.getyourguide.com/antibes-l1234/`) is used as the
  destination link itself, and is correct wherever the guest connects from. This
  is the reliable option, and the host sheet says so.

A value counts as a URL when it declares a scheme, starts with `www.`, names a
GetYourGuide host, or pairs a plausible host with a path — a place name never
does. A pasted URL goes through the **same allowlist and normalization as the
curated links**: `getyourguide.com` (any subdomain) or `gyg.me` only, partner id
set or replaced, anything else refused at save with
`activities_url_not_getyourguide` and flagged in the host sheet before saving.

The button label names the place: it comes from the destination slug
(`/cannes-l15/` → `Cannes`, `/aix-en-provence-l1234/` → `Aix en Provence`), then
from the city in the address when the URL carries no slug (a `gyg.me` short
link), then from a neutral label (`guest.activities.browseLabel`).

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
changing it means a module release. **Emptying it stays legal**: links then
render with no partner parameter at all, and nothing else changes.

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
