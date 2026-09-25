# Changelog

## [0.5.1](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.5.0...wifi-guest-v0.5.1) (2026-09-25)


### Bug Fixes

* **deps:** build every module on portaki-sdk 8.0.1 ([734516f](https://github.com/PortakiApp/portaki-modules/commit/734516fb419ac0060f1c54a415c221316f85332b))

## [0.5.0](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.4.1...wifi-guest-v0.5.0) (2026-09-24)


### Features

* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **wifi-guest:** check network before publish ([5f9e8f1](https://github.com/PortakiApp/portaki-modules/commit/5f9e8f1cdd0dd9d0c9113bbff7e47726bb2ff7d7))

## [0.4.1](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.4.0...wifi-guest-v0.4.1) (2026-09-24)


### Bug Fixes

* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))

## [0.4.0](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.3.0...wifi-guest-v0.4.0) (2026-09-23)


### Features

* **wifi-guest:** add catalogue listing ([251f1a9](https://github.com/PortakiApp/portaki-modules/commit/251f1a9f3f54471d8df8cc7ac89504eb8e06f018))

## [0.3.0](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.2.3...wifi-guest-v0.3.0) (2026-09-23)


### Features

* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))

## [0.2.3](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.2.2...wifi-guest-v0.2.3) (2026-09-17)


### Bug Fixes

* **modules:** use host clock, ban native clock ([6c660c0](https://github.com/PortakiApp/portaki-modules/commit/6c660c033429260fe8dfaff7be808b80fffe49fa))

## [0.2.2](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.2.1...wifi-guest-v0.2.2) (2026-09-16)


### Bug Fixes

* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))

## [0.2.1](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.2.0...wifi-guest-v0.2.1) (2026-09-12)


### Bug Fixes

* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))

## [0.2.0](https://github.com/PortakiApp/portaki-modules/compare/wifi-guest-v0.1.4...wifi-guest-v0.2.0) (2026-09-12)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **ical-sync:** add host calendar import module ([b2324cd](https://github.com/PortakiApp/portaki-modules/commit/b2324cd5328a90052c3a54a6496bbd8875184467))
* **modules:** add events, nuki, and wifi-guest ([192517e](https://github.com/PortakiApp/portaki-modules/commit/192517e9a071c7c2565bce7c2fc3b09e482bc177))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))


### Bug Fixes

* **i18n:** fill weather keys; harden wifi reveal ([d7cce9c](https://github.com/PortakiApp/portaki-modules/commit/d7cce9c846a1c070a68a475005bcf5d1039087ed))
* **wifi-guest:** flatten host drawer SDUI form ([cfcd54d](https://github.com/PortakiApp/portaki-modules/commit/cfcd54de230125dee5b4884e864441235049c8bb))

## [Unreleased]

## [0.1.3] — 2026-07-26

### Changed

- Host drawer SDUI matches Portaki Dashboard `sduiForm` for wifi-guest: warning alert +
  flat labeled fields (no nested Cards). Drawer chrome stays in the host shell.
- Optional `connection_steps` field (design drawer) persisted in config and shown on guest.

## [0.1.2]

### Added

- Initial `wifi-guest` module: host config, guest SDUI, reveal policy, `emailContext` (`wifiName`).
