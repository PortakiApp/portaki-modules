# Changelog

## [0.3.1](https://github.com/PortakiApp/portaki-modules/compare/sections-v0.3.0...sections-v0.3.1) (2026-09-12)


### Bug Fixes

* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))

## [0.3.0](https://github.com/PortakiApp/portaki-modules/compare/sections-v0.2.5...sections-v0.3.0) (2026-09-12)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **sections:** align host SDUI to design ([0e7e769](https://github.com/PortakiApp/portaki-modules/commit/0e7e769f5580ee2611848ca9c4cc4d1629704677))
* **sections:** align welcome module with the design ([4934945](https://github.com/PortakiApp/portaki-modules/commit/4934945f6488991ba54003d4b7dc39039c40eb3d))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))

## [0.2.5](https://github.com/PortakiApp/portaki-modules/compare/sections-v0.2.4...sections-v0.2.5) (2026-07-28)


### Bug Fixes

* **host:** drop the visible internal `lang` field from the section detail form — the editing locale comes only from the top language selector (`ctx.locale`)
* **guest:** stop duplicating the section title inside the card when a single section is shown (header already renders it)


### Code Refactoring

* **commands:** remove legacy `title_fr` / `title_en` / `body_markdown_fr` / `body_markdown_en` fields — the module edits a single active locale


## [0.2.3](https://github.com/PortakiApp/portaki-modules/compare/sections-v0.2.2...sections-v0.2.3) (2026-07-24)


### Features

* align host SDUI to design `sections-editor-v1` (master-detail list, TipTap editor, add/cancel/save)


## [0.2.0](https://github.com/PortakiApp/portaki-modules/compare/sections-v0.1.0...sections-v0.2.0) (2026-07-21)


### Features

* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
