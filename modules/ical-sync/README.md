# ical-sync

Official Portaki **host-only** module: import stays from iCal / Airbnb (and other `.ics`) calendar export URLs.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`ical-sync`

## Audience

Host dashboard only — no guest booklet surfaces.

## Host config surface

`property-module-sheet` — config cards open in the module configure sheet (not a listing sidebar tab).

## Scheduled / manual sync

Manifest `hostScheduledSync` uses the platform-fetch path:

1. Query `listSources` → feed URLs  
2. Platform HTTPS-fetches each `.ics` body  
3. Query `applyFeeds` → parses VEVENT rows + updates `last_sync_at` / `sync_summary`  
4. Platform imports stays (`guestName`, `checkInAt`, `checkOutAt`, `icalUid`, …)

Manual trigger: `POST /api/v1/properties/{id}/modules/ical-sync/sync`.

## Capabilities

| Capability | Role |
|------------|------|
| `core.storage` | **Required** — KV config |
| `core.ical.import` | **Required** — plan allowance for calendar import |

## KV config

```json
{
  "ical_url_primary": "https://…/calendar.ics",
  "ical_url_secondary": "",
  "last_sync_at": "2026-07-23T08:12:00Z",
  "sync_summary": "3 stay(s) · 1 feed(s) ok · 0 feed(s) failed"
}
```

`ical_url_primary` is mirrored to the property `icalUrl` field by the platform.

## Queries / commands

| Op | Kind | Role |
|----|------|------|
| `getConfig` | query | Read config |
| `updateConfig` | command | Save feed URLs |
| `listSources` | query | Sources for platform fetch |
| `applyFeeds` | query | Parse ICS bodies → stay rows |

## Development

```bash
cargo test -p ical-sync
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
