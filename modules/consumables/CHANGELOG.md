# Changelog

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
