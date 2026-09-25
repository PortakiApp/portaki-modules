# Changelog

## [1.0.1](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v1.0.0...pre-arrival-form-v1.0.1) (2026-09-25)


### Bug Fixes

* **deps:** build every module on portaki-sdk 8.0.1 ([734516f](https://github.com/PortakiApp/portaki-modules/commit/734516fb419ac0060f1c54a415c221316f85332b))

## [1.0.0](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.9.1...pre-arrival-form-v1.0.0) (2026-09-24)


### ⚠ BREAKING CHANGES

* **pre-arrival-form:** fresh installs only. A database where pre-arrival-form is already installed keeps its old revisions; do not upgrade it to this version.
* **checklist:** fresh installs only. A database where checklist is already installed keeps its old revisions; do not upgrade it to this version.

### Features

* **pre-arrival-form:** add publish readiness check ([b0cc38f](https://github.com/PortakiApp/portaki-modules/commit/b0cc38fe956d957e6f4ea27d8ba9959bd4d28d9a))
* **pre-arrival-form:** open guest operations explicitly ([d7a54a4](https://github.com/PortakiApp/portaki-modules/commit/d7a54a45ee43c1d51358b55050f799aadfb1e55c))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Miscellaneous

* **checklist:** squash migrations into v1 ([39a112a](https://github.com/PortakiApp/portaki-modules/commit/39a112a3bbd4204004e07621e7664e49663babb8))
* **pre-arrival-form:** squash migrations into v1 ([6d68cad](https://github.com/PortakiApp/portaki-modules/commit/6d68cad684313cddeb5649dd69579bd57e71b3d7))

## [0.9.1](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.9.0...pre-arrival-form-v0.9.1) (2026-09-24)


### Bug Fixes

* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))

## [0.9.0](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.8.0...pre-arrival-form-v0.9.0) (2026-09-23)


### Features

* **pre-arrival-form:** add catalogue listing ([1542a7a](https://github.com/PortakiApp/portaki-modules/commit/1542a7aa2656c687de6a9983018103c1664e639c))

## [0.8.0](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.7.3...pre-arrival-form-v0.8.0) (2026-09-23)


### Features

* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))

## [0.7.3](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.7.2...pre-arrival-form-v0.7.3) (2026-09-17)


### Bug Fixes

* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))
* **pre-arrival-form:** add missing guest nav label ([9028e5e](https://github.com/PortakiApp/portaki-modules/commit/9028e5e60512c74f91cbd95a5b4f17fb3e81e6b5))

## [0.7.2](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.7.1...pre-arrival-form-v0.7.2) (2026-09-16)


### Bug Fixes

* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))

## [0.7.1](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.7.0...pre-arrival-form-v0.7.1) (2026-09-12)


### Bug Fixes

* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))

## [0.7.0](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v0.6.2...pre-arrival-form-v0.7.0) (2026-09-12)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **email:** localize module guest emails ([d7186a0](https://github.com/PortakiApp/portaki-modules/commit/d7186a0b21941d476d12d9a6c14e3965f5c4d057))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** opt into timed email catch-up ([8f826f4](https://github.com/PortakiApp/portaki-modules/commit/8f826f4198b6fc6b642045751f50cea3f2633ad4))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **pre-arrival-form:** reopen completed form overlay ([8425961](https://github.com/PortakiApp/portaki-modules/commit/84259611d18817c12592424804c093ba1585bba0))
* **pre-arrival:** add stay-detail host SDUI ([378ddd7](https://github.com/PortakiApp/portaki-modules/commit/378ddd73f4e8a6976245fc7b4615f4a4fc6f66a4))
* **pre-arrival:** align Accueil formalities SDUI ([a3636a7](https://github.com/PortakiApp/portaki-modules/commit/a3636a70277f8984984222c89a128ef9183aa659))
* **pre-arrival:** compose police via HostFragment ([7adcc58](https://github.com/PortakiApp/portaki-modules/commit/7adcc5807b6780b87f69dee389779810437f68ac))
* **pre-arrival:** hide gated card, mail on available ([6b65853](https://github.com/PortakiApp/portaki-modules/commit/6b65853036dd42c830560da807aff5fd0d44f5bd))
* **pre-arrival:** host config and extra questions ([1378311](https://github.com/PortakiApp/portaki-modules/commit/137831166f368107f9d359e00e2ccd70c761052f))
* **pre-arrival:** keep form editable to check-in ([7526478](https://github.com/PortakiApp/portaki-modules/commit/75264785cb66fbf2cbbc3a89896875e0cab02569))
* **pre-arrival:** opt into config-update catch-up ([6017c5a](https://github.com/PortakiApp/portaki-modules/commit/6017c5a6f99c9d6db46297ed898b5ddc142cb673))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **modules:** bump lost-found and pre-arrival pins ([6e2d26d](https://github.com/PortakiApp/portaki-modules/commit/6e2d26dfadbaab7e358420e8ce6f5fe11dccfdf4))
* **modules:** rattraper le champ booking_channel du SDK ([c74e36c](https://github.com/PortakiApp/portaki-modules/commit/c74e36c417156ebc742e8bc72dc0ec22c3883219))
* **pre-arrival-form:** restore JSON comma after version ([5cf020f](https://github.com/PortakiApp/portaki-modules/commit/5cf020f74992d7636f8f027b625359a1b3fcf7f7))
* **pre-arrival:** drop form Card; bump 0.4.3 ([bab64fa](https://github.com/PortakiApp/portaki-modules/commit/bab64fa91f5cbe4c76721ce0ff80728de231e3fc))
* **pre-arrival:** keep false toggles on updateConfig ([b3432af](https://github.com/PortakiApp/portaki-modules/commit/b3432af12c76ae705119c94b8edfa8c46ca45436))
* **pre-arrival:** omit gated form soon teaser ([c150328](https://github.com/PortakiApp/portaki-modules/commit/c1503287aa531d0296d23f18a738174f5572651d))
* **pre-arrival:** show question icons and borders ([6273aff](https://github.com/PortakiApp/portaki-modules/commit/6273affb05a07f730ba6fea6fb7e2d107aa52b84))

## [0.6.1]

### Added

- Manifest `emails[].catchUpOnConfigUpdate` for `form-available` — platform
  promotes draft KV and redispatches `sendFormAvailable` after host
  `updateConfig` (workspace-tab Save without property Publish).

## [0.6.0]

### Added

- Manifest `emails[]` for `form-available`: `command: sendFormAvailable`,
  `catchUpOnPropertyPublish`, `dispatchOnStayCreated`. Platform redispatches on
  property publish / stay created; module gate + delivery dedup unchanged.

## [0.5.1]

### Changed

- Guest form stays editable after submit until stay check-in (prefilled fields + resubmit).
- After check-in, overlay is read-only; `submit` rejects with `form_locked_after_checkin`.

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
