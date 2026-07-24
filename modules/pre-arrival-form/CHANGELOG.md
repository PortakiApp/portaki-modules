# Changelog

## [0.3.0]

### Added

- Host config SDUI (`prearrival-editor-v1`): when-to-show ChoiceList + question ToggleRow grid.
- KV `config` (`show_when`, `ask_*` question flags) persisted via `updateConfig`.
- Guest form respects enabled questions and `show_when` timing (48 h before / check-in day).
- Extra response fields: guest count, special needs, ID document (schema v2).

## [0.2.0]

### Added

- Host `stay-detail` surface (`pathSegment`: `stay`) with `render_host_stay` SDUI for stay detail.

## [0.1.0]

### Added

- Initial `pre-arrival-form` module: guest form, host workspace tab, ETA / occasion / allergies.
