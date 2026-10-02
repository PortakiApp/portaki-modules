# Changelog

## [1.2.0](https://github.com/PortakiApp/portaki-modules/compare/sections-v1.1.0...sections-v1.2.0) (2026-10-02)


### Features

* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))
* **sections:** speak the ten languages of the picker ([20fb869](https://github.com/PortakiApp/portaki-modules/commit/20fb869d0023790cd4c62757c98a361b48c0039f))

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/sections-v1.0.0...sections-v1.1.0) (2026-09-29)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** migrate all modules to SDK 2.1 typed APIs ([e5d865b](https://github.com/PortakiApp/portaki-modules/commit/e5d865b74a3295bd7b70cea080b9bf0f6d15b15c))
* **modules:** per-locale texts, access-guide redesign ([3f0296a](https://github.com/PortakiApp/portaki-modules/commit/3f0296a6bb3128d8a0ca485db344dc9e49ce5aac))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **sections:** add catalogue listing ([aab8633](https://github.com/PortakiApp/portaki-modules/commit/aab8633ee6fc498a4a271bab7e0ed77d3adb228b))
* **sections:** align host SDUI to design ([0e7e769](https://github.com/PortakiApp/portaki-modules/commit/0e7e769f5580ee2611848ca9c4cc4d1629704677))
* **sections:** align welcome module with the design ([4934945](https://github.com/PortakiApp/portaki-modules/commit/4934945f6488991ba54003d4b7dc39039c40eb3d))
* **sections:** dispatch examples, scenario tests ([8ca345a](https://github.com/PortakiApp/portaki-modules/commit/8ca345aa9afa560a0f3daac0bd3010414cc58e51))


### Bug Fixes

* **ci:** pin SDK deps to git main for CI ([e723553](https://github.com/PortakiApp/portaki-modules/commit/e7235532d59013d9acdb59cb12840a14d89e74d4))
* **copy:** drop tech jargon from module texts ([f988f54](https://github.com/PortakiApp/portaki-modules/commit/f988f549b38f200687eb66f24443e0f21d057af6))
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
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **sections:** examples for delete and reorder ([52e05b7](https://github.com/PortakiApp/portaki-modules/commit/52e05b747d612f46d807e53765560210a25954c2))

## 1.0.0 (2026-09-29)

First stable release. The catch-all: formatted text the host writes themselves, for everything the other modules do not cover.

* Free-form text sections cover whatever the other modules do not.
* Guests see a preview of the first sections on the booklet's home page, then the full text of each.
* The host writes them with a rich text editor — headings, lists and quotes.
