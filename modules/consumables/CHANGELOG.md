# Changelog

## [1.0.1](https://github.com/PortakiApp/portaki-modules/compare/consumables-v1.0.0...consumables-v1.0.1) (2026-09-25)


### Bug Fixes

* **deps:** build every module on portaki-sdk 8.0.1 ([734516f](https://github.com/PortakiApp/portaki-modules/commit/734516fb419ac0060f1c54a415c221316f85332b))

## [1.0.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v0.5.1...consumables-v1.0.0) (2026-09-24)


### ⚠ BREAKING CHANGES

* **consumables:** fresh installs only. A database where consumables is already installed keeps its old revisions; do not upgrade it to this version.
* **checklist:** fresh installs only. A database where checklist is already installed keeps its old revisions; do not upgrade it to this version.

### Features

* **consumables:** chart shortages per stay and restock rhythm ([d3b9738](https://github.com/PortakiApp/portaki-modules/commit/d3b97388d3f52e3411b95499e707c1576d5155f9))
* **consumables:** open guest operations explicitly ([542ca37](https://github.com/PortakiApp/portaki-modules/commit/542ca370203130687a4d000d0c70cf1a5c060b52))
* **consumables:** serve stats tile and stock detail ([24f05eb](https://github.com/PortakiApp/portaki-modules/commit/24f05ebd1da29c59608e14b6fe49f160e0d1f8e5))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **consumables:** scope reports by property ([7288051](https://github.com/PortakiApp/portaki-modules/commit/728805192f45b427a9371a81cc550c3ce609bd20))


### Miscellaneous

* **checklist:** squash migrations into v1 ([39a112a](https://github.com/PortakiApp/portaki-modules/commit/39a112a3bbd4204004e07621e7664e49663babb8))
* **consumables:** squash migrations into v1 ([8e515a4](https://github.com/PortakiApp/portaki-modules/commit/8e515a4f87e1cac72908d87e66a22c86582fda6c))

## [0.5.1](https://github.com/PortakiApp/portaki-modules/compare/consumables-v0.5.0...consumables-v0.5.1) (2026-09-24)


### Bug Fixes

* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))

## [0.5.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v0.4.0...consumables-v0.5.0) (2026-09-23)


### Features

* **consumables:** add catalogue listing ([d3ae8ac](https://github.com/PortakiApp/portaki-modules/commit/d3ae8ac0ec0636f7b9cb53da46433fc144854872))

## [0.4.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v0.3.2...consumables-v0.4.0) (2026-09-23)


### Features

* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))

## [0.3.2](https://github.com/PortakiApp/portaki-modules/compare/consumables-v0.3.1...consumables-v0.3.2) (2026-09-16)


### Bug Fixes

* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** quote guest text in emails, never fail a saved record ([12d3eef](https://github.com/PortakiApp/portaki-modules/commit/12d3eef9f9f361b024e800cec059f394f7172e4d))

## [0.3.1](https://github.com/PortakiApp/portaki-modules/compare/consumables-v0.3.0...consumables-v0.3.1) (2026-09-12)


### Bug Fixes

* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))

## [0.3.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v0.2.3...consumables-v0.3.0) (2026-09-12)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* **consumables:** add stock shortage Wasm module ([f2ef4d9](https://github.com/PortakiApp/portaki-modules/commit/f2ef4d9d68b1ff67c9069700d9a917db9a9b4e0c))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** move report forms to overlay sheets ([47a31f7](https://github.com/PortakiApp/portaki-modules/commit/47a31f7afa1d032f85a1e988b1b296d6c3ae95d8))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))


### Bug Fixes

* **consumables,events:** use host clock in render path ([3ce36bf](https://github.com/PortakiApp/portaki-modules/commit/3ce36bfd2e59f186d02400c16502554255ee417e))
* **stay-detail:** empty state when no reports ([da86cff](https://github.com/PortakiApp/portaki-modules/commit/da86cffadc4c0f4203d4864190af8cbc8a59aee9))

## [0.1.0] — 2026-07-24

### Added

- Host workspace tab: consumable catalog (indexed slots) + seed defaults + open reports with mark restocked
- Host stay-detail surface for stay-scoped shortage reports
- Host property-stats-card (`stock`) — catalog size + open report count
- Guest `home.card`: pick catalog item + missing/low level + optional note; host email via `host::email::send`
