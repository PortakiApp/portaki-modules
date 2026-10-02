# Changelog

## [1.2.0](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v1.1.0...pre-arrival-form-v1.2.0) (2026-10-02)


### Features

* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))
* **pre-arrival-form:** speak the ten languages of the picker ([98dcd8a](https://github.com/PortakiApp/portaki-modules/commit/98dcd8aa9324120a68ca0dcfaf78d273488c3d73))

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/pre-arrival-form-v1.0.0...pre-arrival-form-v1.1.0) (2026-09-29)


### Features

* **pre-arrival-form:** declare config ([40ad78a](https://github.com/PortakiApp/portaki-modules/commit/40ad78a5603afbfc967f6c3f8ae2f6dc8f851c7e))
* **pre-arrival-form:** dispatch examples, scenario tests ([2484708](https://github.com/PortakiApp/portaki-modules/commit/24847088fd7365bb3a4187c5fc0d30df6ffd4f00))
* **pre-arrival-form:** preview asked fields in email ([afe8447](https://github.com/PortakiApp/portaki-modules/commit/afe8447f3af97ba30c27d2b548fb46f9055f8d9f))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))
* **pre-arrival-form:** keep id document out of event ([f9428a1](https://github.com/PortakiApp/portaki-modules/commit/f9428a15f2d4ddde5e76dfd0cbecbe642a652f57))
* **pre-arrival-form:** no form invite after check-in ([469f706](https://github.com/PortakiApp/portaki-modules/commit/469f706823c21d141e7255952c91c2d048871717))

## 1.0.0 (2026-09-29)

First stable release. The handful of things worth knowing before a guest arrives, asked once, in the booklet.

* Before arrival, guests fill a short form: arrival time, occasion, allergies, number of guests, special needs, ID document.
* An email tells the guest when the form becomes available, and answers land on the stay's detail page.
* The host picks which questions to ask and when the form appears — from booking, 48 h before, or on arrival day.
