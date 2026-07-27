# Changelog

## [0.4.0] — 2026-07-27

### Added

- Every imported stay row now reports the booking platform: `bookingChannel`
  (`airbnb` | `booking` | `abritel-vrbo` | `direct` | `other-platform` |
  `unknown`) and `bookingChannelSignal` (`ical-uid-suffix` | `ical-prodid` |
  `feed-format-declared` | `feed-url-host` | `host-override` | `none`). Both
  keys are always present — an unidentifiable feed emits `unknown` / `none`.
- Detection reads the VEVENT `UID` suffix first, then the calendar `PRODID`
  (previously discarded — it lives in the header, outside any `VEVENT`), then
  what the host declared. `SUMMARY` / `DESCRIPTION` are never consulted.
- Host SDUI: a **booking platform** selector per feed, separate from the
  calendar format, with options driven from the SDK `BookingChannel` catalog.
  Feeds store `channel` + `channel_signal`; a blank choice prefills from the
  feed URL host.

### Changed

- `StayImportRow` now comes from `portaki_sdk::contracts::stay_import` — the
  shape is SDK-owned so every import module emits the same row.
- `parse_stay_rows` takes `&FeedParseContext` instead of a bare `CalendarFormat`
  (crate-internal source change, no wire change).
- `CalendarFormat` is documented as the feed **shape**, not the seller. `Google`
  and `Generic` map to no platform: a Google Calendar mirror resolves to
  `unknown`, never to `google`.

## [0.3.0] — 2026-07-27

### Added

- Host transactional emails via `host::email::send` (module SDUI, FR/EN):
  - `sync-failed` — feed body empty / unreachable (dedup per feed + day)
  - `stay-imported` — single new stay without guest email
  - `sync-summary` — batch digest when several stays are new/updated
- KV `sync_state` snapshot (`icalUid` → dates) to detect new vs updated stays
- Manifest `emails[]` declarations for the three host mails (`onApplyFeeds`)

## [0.2.0] — 2026-07-27

### Added

- Per-feed `format` field: `airbnb` | `booking` | `abritel_vrbo` | `google` | `generic`.
- Host SDUI format selector (+ short help per platform) when adding/editing a calendar URL.
- Format-aware ICS parsing — Airbnb skips « Reserved - Not available » / « Not available » blocks; Booking skips « CLOSED - Not available ».

### Changed

- Existing feeds without `format` migrate on load: URL host detection when safe, otherwise `generic`.
- `listSources.provider` is the declared format (not a URL guess).
- A feed that only contains blocks no longer counts as sync failure.

## [Unreleased]

### Changed

- Config is `calendars[]` only — no more mirrored `ical_url_primary`. Legacy primary/secondary/`feeds_json` still migrate on load.

### Added

- Initial `ical-sync` module: host sheet config, ICS VEVENT parser, `listSources` / `applyFeeds` for platformFetch scheduled sync.
