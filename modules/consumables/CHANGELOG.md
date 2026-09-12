# Changelog

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
