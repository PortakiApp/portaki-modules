# Changelog

## [1.3.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v1.2.1...lost-found-v1.3.0) (2026-10-02)


### Features

* **trails:** the compass for distance, the arrow for ascent ([9aa2e31](https://github.com/PortakiApp/portaki-modules/commit/9aa2e31862e8f226cad5f43ea33b37b84be34e67))

## [1.2.1](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v1.2.0...lost-found-v1.2.1) (2026-10-02)


### Bug Fixes

* **lost-found:** read a window the host form actually sends ([a78a433](https://github.com/PortakiApp/portaki-modules/commit/a78a4337e2517d7cc813c768c1789f537b53833e)), closes [#214](https://github.com/PortakiApp/portaki-modules/issues/214)

## [1.2.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v1.1.0...lost-found-v1.2.0) (2026-10-02)


### Features

* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))
* **lost-found:** let the host set the window and the returns ([d6dac70](https://github.com/PortakiApp/portaki-modules/commit/d6dac70c4605a8f4537d2a98874491cbde4e1efc))
* **lost-found:** speak the ten languages of the picker ([0f58252](https://github.com/PortakiApp/portaki-modules/commit/0f582528967c3b70130215e753877d9a723699bf))

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v1.0.0...lost-found-v1.1.0) (2026-09-29)


### Features

* **lost-found:** dispatch examples, scenario tests ([05868f2](https://github.com/PortakiApp/portaki-modules/commit/05868f2dfe86f46aaf80cdf6d399c30ae10b7791))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **lost-found:** declare host-found guest email ([a143bdb](https://github.com/PortakiApp/portaki-modules/commit/a143bdb2be2a2c6793f4de553f66281f37427cb5))
* **lost-found:** example for updateStatus ([3bb8f9a](https://github.com/PortakiApp/portaki-modules/commit/3bb8f9af804310ee43870c048e71d4fbbfbf9bf6))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))

## 1.0.0 (2026-09-29)

First stable release. A charger left behind is tracked from the moment it is found to the moment it is back with its owner.

* The host declares a found item from the stay's detail page, and the guest is emailed to arrange its return.
* Guests can also report an item lost or found from the booklet, during or after the stay.
* Each item is tracked through to collect, sent and returned, in the property stats.
* Nothing to configure, and an email for every guest report.
