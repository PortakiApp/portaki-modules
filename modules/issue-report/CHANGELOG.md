# Changelog

## [2.0.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v1.0.1...issue-report-v2.0.0) (2026-09-25)


### ⚠ BREAKING CHANGES

* **issue-report:** fresh installs only. A database where issue-report is already installed keeps its old revisions; do not upgrade it to this version.
* **checklist:** fresh installs only. A database where checklist is already installed keeps its old revisions; do not upgrade it to this version.

### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** move report forms to overlay sheets ([47a31f7](https://github.com/PortakiApp/portaki-modules/commit/47a31f7afa1d032f85a1e988b1b296d6c3ae95d8))
* **issue-report:** add catalogue listing ([e6c790c](https://github.com/PortakiApp/portaki-modules/commit/e6c790c8158747efbde1153a7b37b9d2130e571a))
* **issue-report:** add guest in-stay problem reports ([978dbcf](https://github.com/PortakiApp/portaki-modules/commit/978dbcfe26a301d0b6484f136d9ede2730e6e3e5))
* **issue-report:** add property stats card ([e5a8f3c](https://github.com/PortakiApp/portaki-modules/commit/e5a8f3c24593c1f75ec403e28a03021d37d91625))
* **issue-report:** chart stats panels ([7c8b9da](https://github.com/PortakiApp/portaki-modules/commit/7c8b9daea4bd7409ce903ac646fc459e3c85007f))
* **issue-report:** dispatch examples, scenario tests ([4c4ea9b](https://github.com/PortakiApp/portaki-modules/commit/4c4ea9b81c67a6396b07bf39d5bcc1fa2143ea16))
* **issue-report:** let guests attach a photo ([f008016](https://github.com/PortakiApp/portaki-modules/commit/f00801649de385b468c974383d8da684990ee495))
* **issue-report:** move recent reports to stats tab ([889e681](https://github.com/PortakiApp/portaki-modules/commit/889e6815c6aabb2e40160b2b4a72f1ec9a7ab8ce))
* **issue-report:** open guest operations explicitly ([f6b37b7](https://github.com/PortakiApp/portaki-modules/commit/f6b37b78ad716e4fc94447d9c960127a86541e88))
* **issue-report:** resolve reports, real stats ([e8f5e62](https://github.com/PortakiApp/portaki-modules/commit/e8f5e62264cfae94929b5befc0271ee1df777271))
* **issue-report:** serve stats tile and feed detail ([2db29ca](https://github.com/PortakiApp/portaki-modules/commit/2db29ca2c610049631967a657a439afc56d74d41))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** own mail and lost-found host SDUI ([a62e5be](https://github.com/PortakiApp/portaki-modules/commit/a62e5be10fa3049690cdb6185975d7175933e83e))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))
* **stats:** give Stat tiles their icons ([e4c722a](https://github.com/PortakiApp/portaki-modules/commit/e4c722a4dbb4ad1d5aafa215e6d991b0ecfde7ac))
* **stats:** image icon on photo tiles ([c8b42c6](https://github.com/PortakiApp/portaki-modules/commit/c8b42c60b78c44266ae9620ada43b61808576b7f))


### Bug Fixes

* **clippy:** use sort_by_key for newest-first ([873eebb](https://github.com/PortakiApp/portaki-modules/commit/873eebb544d485e5f7107a7df1441db225b07c85))
* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))
* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))
* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **issue-report:** type submit category as enum ([77c44c5](https://github.com/PortakiApp/portaki-modules/commit/77c44c58dd6c15a6bfd2a844123dec061a5f5270))
* **modules:** quote guest text in emails, never fail a saved record ([12d3eef](https://github.com/PortakiApp/portaki-modules/commit/12d3eef9f9f361b024e800cec059f394f7172e4d))


### Miscellaneous

* **checklist:** squash migrations into v1 ([39a112a](https://github.com/PortakiApp/portaki-modules/commit/39a112a3bbd4204004e07621e7664e49663babb8))
* **issue-report:** squash migrations into v1 ([4454b66](https://github.com/PortakiApp/portaki-modules/commit/4454b6678b2d4f04012ed3cf46bb818e00d51350))

## [1.0.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v0.6.0...issue-report-v1.0.0) (2026-09-24)


### ⚠ BREAKING CHANGES

* **issue-report:** fresh installs only. A database where issue-report is already installed keeps its old revisions; do not upgrade it to this version.
* **checklist:** fresh installs only. A database where checklist is already installed keeps its old revisions; do not upgrade it to this version.

### Features

* **issue-report:** open guest operations explicitly ([f6b37b7](https://github.com/PortakiApp/portaki-modules/commit/f6b37b78ad716e4fc94447d9c960127a86541e88))
* **issue-report:** serve stats tile and feed detail ([2db29ca](https://github.com/PortakiApp/portaki-modules/commit/2db29ca2c610049631967a657a439afc56d74d41))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Bug Fixes

* **issue-report:** type submit category as enum ([77c44c5](https://github.com/PortakiApp/portaki-modules/commit/77c44c58dd6c15a6bfd2a844123dec061a5f5270))


### Miscellaneous

* **checklist:** squash migrations into v1 ([39a112a](https://github.com/PortakiApp/portaki-modules/commit/39a112a3bbd4204004e07621e7664e49663babb8))
* **issue-report:** squash migrations into v1 ([4454b66](https://github.com/PortakiApp/portaki-modules/commit/4454b6678b2d4f04012ed3cf46bb818e00d51350))

## [0.6.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v0.5.0...issue-report-v0.6.0) (2026-09-24)


### Features

* **issue-report:** move recent reports to stats tab ([889e681](https://github.com/PortakiApp/portaki-modules/commit/889e6815c6aabb2e40160b2b4a72f1ec9a7ab8ce))


### Bug Fixes

* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))

## [0.5.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v0.4.0...issue-report-v0.5.0) (2026-09-23)


### Features

* **issue-report:** add catalogue listing ([e6c790c](https://github.com/PortakiApp/portaki-modules/commit/e6c790c8158747efbde1153a7b37b9d2130e571a))

## [0.4.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v0.3.2...issue-report-v0.4.0) (2026-09-23)


### Features

* **issue-report:** add property stats card ([e5a8f3c](https://github.com/PortakiApp/portaki-modules/commit/e5a8f3c24593c1f75ec403e28a03021d37d91625))
* **issue-report:** chart stats panels ([7c8b9da](https://github.com/PortakiApp/portaki-modules/commit/7c8b9daea4bd7409ce903ac646fc459e3c85007f))
* **issue-report:** let guests attach a photo ([f008016](https://github.com/PortakiApp/portaki-modules/commit/f00801649de385b468c974383d8da684990ee495))
* **issue-report:** resolve reports, real stats ([e8f5e62](https://github.com/PortakiApp/portaki-modules/commit/e8f5e62264cfae94929b5befc0271ee1df777271))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))

## [0.3.2](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v0.3.1...issue-report-v0.3.2) (2026-09-16)


### Bug Fixes

* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** quote guest text in emails, never fail a saved record ([12d3eef](https://github.com/PortakiApp/portaki-modules/commit/12d3eef9f9f361b024e800cec059f394f7172e4d))

## [0.3.1](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v0.3.0...issue-report-v0.3.1) (2026-09-12)


### Bug Fixes

* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))

## [0.3.0](https://github.com/PortakiApp/portaki-modules/compare/issue-report-v0.2.3...issue-report-v0.3.0) (2026-09-12)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** move report forms to overlay sheets ([47a31f7](https://github.com/PortakiApp/portaki-modules/commit/47a31f7afa1d032f85a1e988b1b296d6c3ae95d8))
* **issue-report:** add guest in-stay problem reports ([978dbcf](https://github.com/PortakiApp/portaki-modules/commit/978dbcfe26a301d0b6484f136d9ede2730e6e3e5))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** own mail and lost-found host SDUI ([a62e5be](https://github.com/PortakiApp/portaki-modules/commit/a62e5be10fa3049690cdb6185975d7175933e83e))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))


### Bug Fixes

* **clippy:** use sort_by_key for newest-first ([873eebb](https://github.com/PortakiApp/portaki-modules/commit/873eebb544d485e5f7107a7df1441db225b07c85))

## [Unreleased]

### Added

- Initial `issue-report` module: guest home card form, host recent list, `issue-report.submitted` event.
