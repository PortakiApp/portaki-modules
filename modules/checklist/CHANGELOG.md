# Changelog

## [1.2.0](https://github.com/PortakiApp/portaki-modules/compare/checklist-v1.1.0...checklist-v1.2.0) (2026-10-02)


### Features

* **checklist:** group the departure steps and keep the ticks ([6d3b409](https://github.com/PortakiApp/portaki-modules/commit/6d3b409ad732b0da8167096c0fd173ec442ef7cf))
* **checklist:** speak the ten languages of the picker ([d255859](https://github.com/PortakiApp/portaki-modules/commit/d25585980b655d6f0e91115882fe53405314a32d))
* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/checklist-v1.0.0...checklist-v1.1.0) (2026-09-29)


### Features

* **checklist:** block publish until a list has items ([da76ee7](https://github.com/PortakiApp/portaki-modules/commit/da76ee7ae270c3070ffa1e959b95049456a00898))
* **checklist:** dispatch examples, scenario tests ([8b60c84](https://github.com/PortakiApp/portaki-modules/commit/8b60c84ab31be318057bdc7d5cc56b60107bb0ee))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **checklist:** emit restock in own namespace ([5723fbf](https://github.com/PortakiApp/portaki-modules/commit/5723fbfd28b21d8f1de8a77f59e4acfb7dc0664c))
* **checklist:** examples for list, task commands ([41b0c1f](https://github.com/PortakiApp/portaki-modules/commit/41b0c1f1960ca12347efb907971232649373f96e))
* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))

## 1.0.0 (2026-09-29)

First stable release. One module for the lists that run a stay: what the guest ticks before leaving, and what the host's team ticks after.

* Guests tick their departure list in the booklet, shown at the time the host chooses.
* Cleaning and inspection lists become dated tasks around each stay, with an assignee, a deadline and an optional required photo.
* The host starts from one of four templates or an empty list, and follows Checklist and Cleaning views in Statistics.
