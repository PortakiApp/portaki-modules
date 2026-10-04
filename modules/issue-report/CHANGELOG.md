# Changelog

## [1.4.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v1.3.0...issue-report-v1.4.0) (2026-10-04)


### Features

* **trails:** le tracé d'un itinéraire, lu du GPX de l'hôte ([0fae471](https://github.com/PortakiApp/portaki-modules/commit/0fae471f7ef8c106306f0d7b782d22d9ebbac9ba))

## [1.3.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v1.2.0...issue-report-v1.3.0) (2026-10-02)


### Features

* **issue-report:** say not to wait when it cannot wait ([e731c1f](https://github.com/PortakiApp/portaki-modules/commit/e731c1f1831deadb9378ab442f2ffe20f44b0c9d))
* **trails:** the compass for distance, the arrow for ascent ([9aa2e31](https://github.com/PortakiApp/portaki-modules/commit/9aa2e31862e8f226cad5f43ea33b37b84be34e67))

## [1.2.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v1.1.0...issue-report-v1.2.0) (2026-10-02)


### Features

* **facility-hours:** today's hours in two stay emails ([64f467f](https://github.com/PortakiApp/portaki-modules/commit/64f467fbd8d64524bfb696c5f09290ef1076893f))
* **issue-report:** let the host pick the form categories ([3522ce9](https://github.com/PortakiApp/portaki-modules/commit/3522ce9104ae2fd8213f0fbacd121a89808e8840))
* **issue-report:** speak the ten languages of the picker ([1a53b34](https://github.com/PortakiApp/portaki-modules/commit/1a53b348c47a4350002a1e40751396b9b9a7d0e1))

## [1.1.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v1.0.0...issue-report-v1.1.0) (2026-09-29)


### Features

* **issue-report:** dispatch examples, scenario tests ([4c4ea9b](https://github.com/PortakiApp/portaki-modules/commit/4c4ea9b81c67a6396b07bf39d5bcc1fa2143ea16))
* **issue-report:** show category and photo as blocks ([6716217](https://github.com/PortakiApp/portaki-modules/commit/6716217d19e1878bc7d7e6284443a9eaf57a62a3))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **deps:** bump portaki-sdk to 8.11.0 ([f7fded1](https://github.com/PortakiApp/portaki-modules/commit/f7fded1fa42013f963eecdeaa8280c89c1803172))
* **deps:** bump portaki-sdk to 8.11.1 ([bdce7fc](https://github.com/PortakiApp/portaki-modules/commit/bdce7fcf958ed3f1935e5f12662fe71ddc58ca9e))
* **deps:** bump portaki-sdk to 8.12.0 ([b6ef90f](https://github.com/PortakiApp/portaki-modules/commit/b6ef90f6cb2324b9ee4363939a52213e81ba8424))
* **deps:** bump portaki-sdk to 8.8.1 ([474417d](https://github.com/PortakiApp/portaki-modules/commit/474417d9ac43536f465f10afbc25e46723aface5))
* **deps:** bump portaki-sdk to 9.0.0 ([e5f9c7f](https://github.com/PortakiApp/portaki-modules/commit/e5f9c7f53100ce123c34af4aae4f6aee91b1569c))
* **issue-report:** example for resolve ([0eda02e](https://github.com/PortakiApp/portaki-modules/commit/0eda02ea6ddc2aa9854d57a9aecc7bfaa9864c11))
* **modules:** republish signed artifacts ([26d3c6e](https://github.com/PortakiApp/portaki-modules/commit/26d3c6eb7ed41c0be9988f783fa68146e11e45fe))

## 1.0.0 (2026-09-29)

First stable release. A guest reports a problem from the booklet and the host hears about it the same minute, with a photo.

* Guests report a problem by category — appliance, cleanliness, noise, access — with a summary, details and a photo.
* The host is emailed straight away, and closes the report from the property stats.
* The same page tracks volume, categories and time to resolve, with the 20 latest reports.
* Nothing to configure: the module works as soon as it is installed.
