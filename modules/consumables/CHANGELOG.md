# Changelog

## [2.4.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v2.3.0...consumables-v2.4.0) (2026-10-04)


### Features

* **pre-arrival-form, consumables:** des choix, pas des champs libres ([f0b5e16](https://github.com/PortakiApp/portaki-modules/commit/f0b5e163dc29eaafe52759cd4eba97fc836b6a60))
* **trails:** le tracé d'un itinéraire, lu du GPX de l'hôte ([0fae471](https://github.com/PortakiApp/portaki-modules/commit/0fae471f7ef8c106306f0d7b782d22d9ebbac9ba))

## [2.3.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v2.2.0...consumables-v2.3.0) (2026-10-02)


### Features

* **consumables:** let the catalog grid grow past its eight tiles ([a912371](https://github.com/PortakiApp/portaki-modules/commit/a91237118169d0b3eb8f21f6d92a50266ab6ffa1))
* **trails:** the compass for distance, the arrow for ascent ([9aa2e31](https://github.com/PortakiApp/portaki-modules/commit/9aa2e31862e8f226cad5f43ea33b37b84be34e67))

## [2.2.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v2.1.0...consumables-v2.2.0) (2026-10-02)


### Features

* **consumables:** a grid the guest can tick more than once ([543c253](https://github.com/PortakiApp/portaki-modules/commit/543c253d11678ab9d479fcbc83e4ff00b0dacd2b))
* **consumables:** speak the ten languages of the picker ([6378481](https://github.com/PortakiApp/portaki-modules/commit/63784810392c2e4e28896a0c1817de078e68ece5))
* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))

## [2.1.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v2.0.0...consumables-v2.1.0) (2026-10-02)


### Features

* **consumables:** a grid the guest can tick more than once ([543c253](https://github.com/PortakiApp/portaki-modules/commit/543c253d11678ab9d479fcbc83e4ff00b0dacd2b))
* **consumables:** show item and level as rows ([1ea2703](https://github.com/PortakiApp/portaki-modules/commit/1ea270399794efd993317fede1cd5a14b6d35497))
* **consumables:** speak the ten languages of the picker ([6378481](https://github.com/PortakiApp/portaki-modules/commit/63784810392c2e4e28896a0c1817de078e68ece5))
* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))


### Bug Fixes

* **consumables:** example for updateStatus ([51939d6](https://github.com/PortakiApp/portaki-modules/commit/51939d633a43f522966e94f7dde8eb1bbe31c72f))
* **consumables:** no label key in example input ([fd1daa7](https://github.com/PortakiApp/portaki-modules/commit/fd1daa77903898abe252b7700cf1d0a39979b916))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))

## [2.0.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v1.2.0...consumables-v2.0.0) (2026-09-30)


### ⚠ BREAKING CHANGES

* **pre-arrival-form:** fresh installs only. A database where pre-arrival-form is already installed keeps its old revisions; do not upgrade it to this version.
* **consumables:** fresh installs only. A database where consumables is already installed keeps its old revisions; do not upgrade it to this version.
* **checklist:** fresh installs only. A database where checklist is already installed keeps its old revisions; do not upgrade it to this version.

### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* **consumables:** a grid the guest can tick more than once ([543c253](https://github.com/PortakiApp/portaki-modules/commit/543c253d11678ab9d479fcbc83e4ff00b0dacd2b))
* **consumables:** add catalogue listing ([d3ae8ac](https://github.com/PortakiApp/portaki-modules/commit/d3ae8ac0ec0636f7b9cb53da46433fc144854872))
* **consumables:** add stock shortage Wasm module ([f2ef4d9](https://github.com/PortakiApp/portaki-modules/commit/f2ef4d9d68b1ff67c9069700d9a917db9a9b4e0c))
* **consumables:** block publish on empty catalog ([e0f435e](https://github.com/PortakiApp/portaki-modules/commit/e0f435ebfb95c32c9a89c9ddd1fcbfc201e4a9e4))
* **consumables:** chart shortages per stay and restock rhythm ([d3b9738](https://github.com/PortakiApp/portaki-modules/commit/d3b97388d3f52e3411b95499e707c1576d5155f9))
* **consumables:** dispatch examples, scenario tests ([4fb36a4](https://github.com/PortakiApp/portaki-modules/commit/4fb36a44e9c5ef3dab3c12b82c8ee627f179446b))
* **consumables:** open guest operations explicitly ([542ca37](https://github.com/PortakiApp/portaki-modules/commit/542ca370203130687a4d000d0c70cf1a5c060b52))
* **consumables:** serve stats tile and stock detail ([24f05eb](https://github.com/PortakiApp/portaki-modules/commit/24f05ebd1da29c59608e14b6fe49f160e0d1f8e5))
* **consumables:** show item and level as rows ([1ea2703](https://github.com/PortakiApp/portaki-modules/commit/1ea270399794efd993317fede1cd5a14b6d35497))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** move report forms to overlay sheets ([47a31f7](https://github.com/PortakiApp/portaki-modules/commit/47a31f7afa1d032f85a1e988b1b296d6c3ae95d8))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **consumables,events:** use host clock in render path ([3ce36bf](https://github.com/PortakiApp/portaki-modules/commit/3ce36bfd2e59f186d02400c16502554255ee417e))
* **consumables:** example for updateStatus ([51939d6](https://github.com/PortakiApp/portaki-modules/commit/51939d633a43f522966e94f7dde8eb1bbe31c72f))
* **consumables:** no label key in example input ([fd1daa7](https://github.com/PortakiApp/portaki-modules/commit/fd1daa77903898abe252b7700cf1d0a39979b916))
* **consumables:** scope reports by property ([7288051](https://github.com/PortakiApp/portaki-modules/commit/728805192f45b427a9371a81cc550c3ce609bd20))
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
* **modules:** quote guest text in emails, never fail a saved record ([12d3eef](https://github.com/PortakiApp/portaki-modules/commit/12d3eef9f9f361b024e800cec059f394f7172e4d))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **stay-detail:** empty state when no reports ([da86cff](https://github.com/PortakiApp/portaki-modules/commit/da86cffadc4c0f4203d4864190af8cbc8a59aee9))


### Miscellaneous

* **checklist:** squash migrations into v1 ([39a112a](https://github.com/PortakiApp/portaki-modules/commit/39a112a3bbd4204004e07621e7664e49663babb8))
* **consumables:** squash migrations into v1 ([8e515a4](https://github.com/PortakiApp/portaki-modules/commit/8e515a4f87e1cac72908d87e66a22c86582fda6c))
* **pre-arrival-form:** squash migrations into v1 ([6d68cad](https://github.com/PortakiApp/portaki-modules/commit/6d68cad684313cddeb5649dd69579bd57e71b3d7))

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/consumables-v1.0.0...consumables-v1.1.0) (2026-09-29)


### Features

* **consumables:** block publish on empty catalog ([e0f435e](https://github.com/PortakiApp/portaki-modules/commit/e0f435ebfb95c32c9a89c9ddd1fcbfc201e4a9e4))
* **consumables:** dispatch examples, scenario tests ([4fb36a4](https://github.com/PortakiApp/portaki-modules/commit/4fb36a44e9c5ef3dab3c12b82c8ee627f179446b))
* **consumables:** show item and level as rows ([1ea2703](https://github.com/PortakiApp/portaki-modules/commit/1ea270399794efd993317fede1cd5a14b6d35497))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **consumables:** example for updateStatus ([51939d6](https://github.com/PortakiApp/portaki-modules/commit/51939d633a43f522966e94f7dde8eb1bbe31c72f))
* **consumables:** no label key in example input ([fd1daa7](https://github.com/PortakiApp/portaki-modules/commit/fd1daa77903898abe252b7700cf1d0a39979b916))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))

## 1.0.0 (2026-09-29)

First stable release. Guests flag what has run out, so the host restocks before the next arrival instead of after the complaint.

* Guests report a supply as out or running low — toilet paper, coffee, soap — with an optional note.
* Every report emails the host and lands on the stay's detail page, to be marked restocked after a visit.
* The host manages the property's supply list from a Consumables tab, prefilled with common items.
