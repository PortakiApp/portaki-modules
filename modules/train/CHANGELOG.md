# Changelog

## [2.1.0](https://github.com/PortakiApp/portaki-modules/compare/train-v2.0.1...train-v2.1.0) (2026-10-08)


### Features

* **train:** l'état du train, et la fenêtre du séjour ([1df602b](https://github.com/PortakiApp/portaki-modules/commit/1df602bd9d01f0968a7350107fb8c12314a82eeb))

## [2.0.1](https://github.com/PortakiApp/portaki-modules/compare/train-v2.0.0...train-v2.0.1) (2026-10-07)


### Bug Fixes

* **guest:** scope the prep-card routes per module ([09bb90c](https://github.com/PortakiApp/portaki-modules/commit/09bb90c6f71a7f1529ea9104609915b4d32d0f24))
* **train:** say the day in words, not in ISO ([d79482d](https://github.com/PortakiApp/portaki-modules/commit/d79482dd4b7431164f9ccffdb09fcf1c8d150d03))

## [2.0.0](https://github.com/PortakiApp/portaki-modules/compare/train-v1.4.0...train-v2.0.0) (2026-10-04)


### ⚠ BREAKING CHANGES

* **train:** la gare devient un réglage obligatoire — une installation existante reste incomplète jusqu'à ce que l'hôte la renseigne. Le quai et la distance jusqu'à la gare disparaissent, l'API n'en donnant pas, et les liens de fiche changent de forme (20261004-0812-17654).

### Features

* **train:** read the real departures of the host's station ([5c617c2](https://github.com/PortakiApp/portaki-modules/commit/5c617c22deeaa58b2272da01e069b3594262204d))


### Bug Fixes

* **ci:** pin every module to portaki-sdk 9.8.0 ([a06c2c7](https://github.com/PortakiApp/portaki-modules/commit/a06c2c7bd26763fb6846d0633ab83431c596f383))

## [1.4.0](https://github.com/PortakiApp/portaki-modules/compare/train-v1.3.0...train-v1.4.0) (2026-10-04)


### Features

* **trails:** le tracé d'un itinéraire, lu du GPX de l'hôte ([0fae471](https://github.com/PortakiApp/portaki-modules/commit/0fae471f7ef8c106306f0d7b782d22d9ebbac9ba))
* **train:** le sens, la gare, et la fiche d'un départ ([bda64da](https://github.com/PortakiApp/portaki-modules/commit/bda64da271eea0ae67fc70649ab3f268f2846a12))

## [1.3.0](https://github.com/PortakiApp/portaki-modules/compare/train-v1.2.0...train-v1.3.0) (2026-10-02)


### Features

* **trails:** the compass for distance, the arrow for ascent ([9aa2e31](https://github.com/PortakiApp/portaki-modules/commit/9aa2e31862e8f226cad5f43ea33b37b84be34e67))

## [1.2.0](https://github.com/PortakiApp/portaki-modules/compare/train-v1.1.0...train-v1.2.0) (2026-10-02)


### Features

* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))
* **train:** speak the ten languages of the picker ([994a7d3](https://github.com/PortakiApp/portaki-modules/commit/994a7d35fbdf355260c178773b8966cd477ecba0))

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/train-v1.0.0...train-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** open cards via fullscreen overlay ([530aa1e](https://github.com/PortakiApp/portaki-modules/commit/530aa1e3670d88e906255199bcf736613b9e28c8))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** clear Clippy -D warnings on CI ([9728b67](https://github.com/PortakiApp/portaki-modules/commit/9728b67d829b9b31ffee4534051cd1f64e855b5f))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))

## 1.0.0 (2026-09-29)

First stable release. A departure board for the nearest station, at a glance in the booklet.

* A home card shows the next departures from the nearest station, across destinations.
* A detail screen adds a from/to header, destination filter chips and the next departures for the one picked.
* Nothing for the host to set: the station and its destinations ship with the module.
* Station and schedules are static for now; reading them from config or a carrier connector comes next.
