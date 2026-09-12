# Changelog

## [0.5.1](https://github.com/PortakiApp/portaki-modules/compare/ical-sync-v0.5.0...ical-sync-v0.5.1) (2026-09-12)


### Bug Fixes

* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))

## [0.5.0](https://github.com/PortakiApp/portaki-modules/compare/ical-sync-v0.4.1...ical-sync-v0.5.0) (2026-09-12)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **ical-sync:** add host sync emails via Wasm ([6fdcb49](https://github.com/PortakiApp/portaki-modules/commit/6fdcb499e0a344d9631daca1f908fe4957194d3e))
* **ical-sync:** declare per-feed ICS format ([a14f67f](https://github.com/PortakiApp/portaki-modules/commit/a14f67fd65ff6bba8c2f4103811a8f44b38981b4))
* **ical-sync:** detect booking channel per stay row ([7609290](https://github.com/PortakiApp/portaki-modules/commit/76092900b9179dfa35b209827aab7add905d0832))
* **ical-sync:** merge format + platform into one selector ([a678eef](https://github.com/PortakiApp/portaki-modules/commit/a678eef70b06a84f2d811258dd5770096d45c05c))
* **ical-sync:** multi-calendar sync and stats card ([ec339e1](https://github.com/PortakiApp/portaki-modules/commit/ec339e1ea24018622e32790e7f0f10fec62e211b))
* **ical-sync:** persist calendars[] only ([67d54b1](https://github.com/PortakiApp/portaki-modules/commit/67d54b11d2462157aae6b48839950c554fa3b479))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))


### Bug Fixes

* **ical-sync:** derive Default for ModuleConfig ([d7b073f](https://github.com/PortakiApp/portaki-modules/commit/d7b073f4bd59f31478f216431e9ddfe7e72f84af))

## [0.4.1] — 2026-07-28

### Changed

- Host SDUI: merged the calendar-format and booking-platform selectors into a
  single **booking platform** picker. The feed shape (format) is deduced instead
  of hand-picked — the feed URL wins (a `google.com` calendar stays a Google
  mirror whoever sold the stay), otherwise the chosen platform implies its export
  shape, else generic. A legacy explicit `format` on write is still honoured, and
  the persisted `channel` / `channel_signal` are unchanged.

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
