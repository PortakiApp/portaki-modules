# Changelog

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
