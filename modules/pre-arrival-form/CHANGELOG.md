# Changelog

## [0.4.2]

### Fixed

- Guest Accueil: when the form is gated by `show_when`, omit the form row / « s’ouvrira bientôt »
  teaser entirely. Police `HostFragment` stays; guest shell hides the card if neither task is visible.

## [0.4.0]

### Added

- Accueil formalities card composes host `regulatory.police-form` via SDUI
  `HostFragment` (no module-named branching in guest/platform).
- Guest surface `guest.form` — fullscreen overlay for the questionnaire.
- Manifest `guestSurfaces`: `role: arrival-formality` +
  `embedsHostFragments: [regulatory.police-form]`.

### Changed

- `home.card` is a checklist composer (design banner) instead of an inline form.

## [0.3.3]

### Fixed

- `updateConfig`: question flags are `Option<bool>` merged into KV — explicit `false`
  sticks, and an empty `{}` payload no longer resets toggles ON via `default_true`.

## [0.3.2]

### Fixed

- Guest `home.card`: when `show_when` gates the form (not yet available), emit EmptyState so the guest shell hides the card entirely.

### Added

- `sendFormAvailable` command — module-owned guest email via `host::email::send` when the form becomes available (tick / stay-created).

## [0.3.1]

### Fixed

- Question ToggleRows emit leading `icon` so host shells render bordered tiles + icon chips (design).

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
