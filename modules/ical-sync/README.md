# ical-sync

Official Portaki **host-only** module: import stays from iCal / Airbnb (and other `.ics`) calendar export URLs.

Part of the [`portaki-modules`](https://github.com/PortakiApp/portaki-modules) monorepo.

## Module id

`ical-sync`

## Audience

Host dashboard only — no guest booklet surfaces.

## Host surfaces

| Type | pathSegment | Role |
|------|-------------|------|
| `property-module-sheet` | `ical-sync` | Config cards in the module configure sheet |
| `property-stats-card` | `calendar-sync` | Tile served by `statsSummary`: time since the last sync, stays imported, conflicts as attention |
| `property-stats-detail` | `calendar-sync` | Detail page: last sync, imports, conflicts, runs per day, stays per calendar |

## Scheduled / manual sync

Manifest `hostScheduledSync` uses the platform-fetch path:

1. Query `listSources` → feed URLs  
2. Platform HTTPS-fetches each `.ics` body  
3. Query `applyFeeds` → parses VEVENT rows + records the run in the `sync_state` KV  
4. Platform imports stays (`guestName`, `checkInAt`, `checkOutAt`, `icalUid`, …)

Manual trigger: `POST /api/v1/properties/{id}/modules/ical-sync/sync`.

## Host emails

Triggered from `applyFeeds` via `host::email::send` (generic `module-host-transactional` shell):

| `email_id` | When |
|------------|------|
| `sync-failed-{feedId}-{YYYY-MM-DD}` | Feed body empty / unreachable |
| `stay-imported-{icalUid}` | Exactly one **new** stay without guest email |
| `sync-summary-{syncAt}` | Several new/updated stays (batch digest) |

Copy lives in `email_i18n/{fr,en}.json`. Dedup is orchestrator-side (module + stay claim + `email_id`).

## Capabilities

| Capability | Role |
|------------|------|
| `core.storage` | **Required** — sync UID snapshot + run history |
| `core.modules.scheduled_sync` | **Required** — the host calls the module on a schedule; the plan sets the cadence |

## Config

Held by the platform (`#[portaki_sdk::config]`, one `structured` field `calendars`, recommended);
the platform takes `updateConfig`. The host form sends each row as strings
(`calendars.N.id|channel|label|url`); `format` and `channel_signal` are resolved on read.
Rows saved by earlier versions keep theirs:

```json
{
  "calendars": [
    {
      "id": "cal-1",
      "url": "https://…/calendar.ics",
      "label": "Airbnb",
      "format": "airbnb",
      "channel": "airbnb",
      "channel_signal": "feed-url-host"
    },
    {
      "id": "cal-2",
      "url": "https://…/other.ics",
      "format": "generic",
      "channel": "direct",
      "channel_signal": "host-override"
    }
  ]
}
```

`calendars` is the only source of truth. Each feed has a `format` (`airbnb` | `booking` | `abritel_vrbo` | `google` | `generic`) — parsing differs (e.g. Airbnb « Reserved » vs « Not available »). Sync fetches every connected URL. A row without `format` gets it from its URL, else from its platform, else `generic`. A KV blob from before the list (`ical_url_primary` / `ical_url_secondary` / `feeds_json`) is mapped onto `calendars` by `#[config(legacy)]`, both for the platform import and while the platform holds no config.

The last run and its summary live in the `sync_state` KV (`lastRunAt`, `summary`), not in the config.

`format` is the feed **shape**; `channel` is **who sold the stay**, from the SDK
`BookingChannel` catalog (`airbnb` | `booking` | `abritel-vrbo` | `direct` |
`other-platform` | `unknown`). They answer different questions: Google Calendar
is a mirror, so `format: "google"` carries no channel. `channel_signal` records
where `channel` came from — `host-override` when the host picked it,
`feed-url-host` when it was prefilled from the URL, `none` when undeclared.

Soft UI cap: 20 calendar rows (`CALENDAR_SLOTS`).

## Booking channel on imported rows

Every row from `applyFeeds` carries `bookingChannel` + `bookingChannelSignal`
(never omitted — an unidentifiable feed emits `unknown` / `none`). First known
match wins:

| Order | Signal | Source |
|-------|--------|--------|
| 1 | `ical-uid-suffix` | VEVENT `UID` domain (`…@airbnb.com`) — survives re-export and mirroring |
| 2 | `ical-prodid` | Calendar `PRODID` — strong when present, cannot split a mixed feed |
| 3 | `host-override` | Host named the platform on the feed config |
| 4 | `feed-format-declared` | Declared `format` names a marketplace |
| 5 | `feed-url-host` | Platform prefilled from the feed URL at config time |
| 6 | `none` | Nothing identified a seller → `unknown` |

`SUMMARY` / `DESCRIPTION` are never consulted — localised strings change without
notice. The feed URL is never read at import: it is wrong on channel-manager
feeds (`beds24.com`, `smoobu.com` mask the origin) and on a Google mirror.

## Queries / commands

| Op | Kind | Role |
|----|------|------|
| `getConfig` | query | Read config |
| `updateConfig` | command | Save calendar list |
| `listSources` | query | Sources for platform fetch |
| `applyFeeds` | query | Parse ICS bodies → stay rows |

## Development

```bash
cargo test -p ical-sync
```

## License

Apache-2.0 — see [LICENSE](../../LICENSE).
