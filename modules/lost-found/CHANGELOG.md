# Changelog

## [2.0.1](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v2.0.0...lost-found-v2.0.1) (2026-09-26)


### Bug Fixes

* **lost-found:** declare host-found guest email ([a143bdb](https://github.com/PortakiApp/portaki-modules/commit/a143bdb2be2a2c6793f4de553f66281f37427cb5))

## [2.0.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v1.0.1...lost-found-v2.0.0) (2026-09-25)


### ⚠ BREAKING CHANGES

* **lost-found:** fresh installs only. A database where lost-found is already installed keeps its old revisions; do not upgrade it to this version.
* **checklist:** fresh installs only. A database where checklist is already installed keeps its old revisions; do not upgrade it to this version.

### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **email:** localize module guest emails ([d7186a0](https://github.com/PortakiApp/portaki-modules/commit/d7186a0b21941d476d12d9a6c14e3965f5c4d057))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** move report forms to overlay sheets ([47a31f7](https://github.com/PortakiApp/portaki-modules/commit/47a31f7afa1d032f85a1e988b1b296d6c3ae95d8))
* **issue-report:** chart stats panels ([7c8b9da](https://github.com/PortakiApp/portaki-modules/commit/7c8b9daea4bd7409ce903ac646fc459e3c85007f))
* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))
* **lost-found:** add catalogue listing ([28ed2c1](https://github.com/PortakiApp/portaki-modules/commit/28ed2c11b46084aa9d8df740b0412671b0e710d6))
* **lost-found:** add guest lost/found reports ([1ccc74f](https://github.com/PortakiApp/portaki-modules/commit/1ccc74f38eaf4258e912e28080f2ed32b8a5d5bd))
* **lost-found:** align host SDUI with design ([285d639](https://github.com/PortakiApp/portaki-modules/commit/285d6395618b8a40bfc089f0d1105852feb6d7cb))
* **lost-found:** dispatch examples, scenario tests ([05868f2](https://github.com/PortakiApp/portaki-modules/commit/05868f2dfe86f46aaf80cdf6d399c30ae10b7791))
* **lost-found:** drop config, serve stats tile ([285c308](https://github.com/PortakiApp/portaki-modules/commit/285c30820005b4297f49e8dd891130b753863445))
* **lost-found:** host found, status, email context ([ba554d6](https://github.com/PortakiApp/portaki-modules/commit/ba554d6498c1ecbc8f4b70dd89781aff6f097ff4))
* **lost-found:** list declared items in stats tab ([f3a734a](https://github.com/PortakiApp/portaki-modules/commit/f3a734a1209a62d3c568221cdd5f83f9d5cb40fb))
* **lost-found:** open guest operations explicitly ([fd7a855](https://github.com/PortakiApp/portaki-modules/commit/fd7a855848740c7088686d794748713a81629568))
* **lost-found:** stay-action create modal surface ([e592d22](https://github.com/PortakiApp/portaki-modules/commit/e592d224269cbfb223d3bb98936a2ce870cdd81e))
* **lost-found:** TipTap create, hide empty stay ([9957d3c](https://github.com/PortakiApp/portaki-modules/commit/9957d3cd277577dfa32cca33d07ea5e2fa0fefbd))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** opt into timed email catch-up ([8f826f4](https://github.com/PortakiApp/portaki-modules/commit/8f826f4198b6fc6b642045751f50cea3f2633ad4))
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
* **modules:** bump lost-found and pre-arrival pins ([6e2d26d](https://github.com/PortakiApp/portaki-modules/commit/6e2d26dfadbaab7e358420e8ce6f5fe11dccfdf4))
* **modules:** quote guest text in emails, never fail a saved record ([12d3eef](https://github.com/PortakiApp/portaki-modules/commit/12d3eef9f9f361b024e800cec059f394f7172e4d))
* **stay-detail:** empty state when no reports ([da86cff](https://github.com/PortakiApp/portaki-modules/commit/da86cffadc4c0f4203d4864190af8cbc8a59aee9))


### Miscellaneous

* **checklist:** squash migrations into v1 ([39a112a](https://github.com/PortakiApp/portaki-modules/commit/39a112a3bbd4204004e07621e7664e49663babb8))
* **lost-found:** squash migrations into v1 ([dec5f31](https://github.com/PortakiApp/portaki-modules/commit/dec5f31ec5d8bd29efedef00ff7eaf1402effadf))

## [1.0.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v0.9.0...lost-found-v1.0.0) (2026-09-24)


### ⚠ BREAKING CHANGES

* **lost-found:** fresh installs only. A database where lost-found is already installed keeps its old revisions; do not upgrade it to this version.
* **checklist:** fresh installs only. A database where checklist is already installed keeps its old revisions; do not upgrade it to this version.

### Features

* **lost-found:** drop config, serve stats tile ([285c308](https://github.com/PortakiApp/portaki-modules/commit/285c30820005b4297f49e8dd891130b753863445))
* **lost-found:** open guest operations explicitly ([fd7a855](https://github.com/PortakiApp/portaki-modules/commit/fd7a855848740c7088686d794748713a81629568))
* **previews:** cover every module with guest surfaces ([0caf3cf](https://github.com/PortakiApp/portaki-modules/commit/0caf3cf90e2935b51940d8af5e9c0c3f7449631a))


### Miscellaneous

* **checklist:** squash migrations into v1 ([39a112a](https://github.com/PortakiApp/portaki-modules/commit/39a112a3bbd4204004e07621e7664e49663babb8))
* **lost-found:** squash migrations into v1 ([dec5f31](https://github.com/PortakiApp/portaki-modules/commit/dec5f31ec5d8bd29efedef00ff7eaf1402effadf))

## [0.9.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v0.8.0...lost-found-v0.9.0) (2026-09-24)


### Features

* **lost-found:** list declared items in stats tab ([f3a734a](https://github.com/PortakiApp/portaki-modules/commit/f3a734a1209a62d3c568221cdd5f83f9d5cb40fb))


### Bug Fixes

* **deps:** build modules against portaki-sdk 6.11 ([18cec5f](https://github.com/PortakiApp/portaki-modules/commit/18cec5fd259c0578979ae06566a86955e8b5d83d))

## [0.8.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v0.7.0...lost-found-v0.8.0) (2026-09-23)


### Features

* **lost-found:** add catalogue listing ([28ed2c1](https://github.com/PortakiApp/portaki-modules/commit/28ed2c11b46084aa9d8df740b0412671b0e710d6))

## [0.7.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v0.6.2...lost-found-v0.7.0) (2026-09-23)


### Features

* **issue-report:** show guest photo as a thumbnail ([bd67919](https://github.com/PortakiApp/portaki-modules/commit/bd679193aa4285ac36b16661c07c657c1d4bdcdb))

## [0.6.2](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v0.6.1...lost-found-v0.6.2) (2026-09-16)


### Bug Fixes

* **deps:** move every module to SDK 5.0.0 ([b359890](https://github.com/PortakiApp/portaki-modules/commit/b359890b7dcadac933be0067d174fbd27a8ba318))
* **deps:** move every module to SDK 5.1.0 ([78889c8](https://github.com/PortakiApp/portaki-modules/commit/78889c82a61c209dc87d10771a8e0e58c659d837))
* **deps:** move every module to SDK 6.0.0 ([fe9d1b2](https://github.com/PortakiApp/portaki-modules/commit/fe9d1b2392a7a1167c9876e77a939efa2ab7a2d1))
* **modules:** quote guest text in emails, never fail a saved record ([12d3eef](https://github.com/PortakiApp/portaki-modules/commit/12d3eef9f9f361b024e800cec059f394f7172e4d))

## [0.6.1](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v0.6.0...lost-found-v0.6.1) (2026-09-12)


### Bug Fixes

* **deps:** declare the SDK in every module manifest ([6e1eb6a](https://github.com/PortakiApp/portaki-modules/commit/6e1eb6a9ab35640062106838a3218ba8bb7d551c))

## [0.6.0](https://github.com/PortakiApp/portaki-modules/compare/lost-found-v0.5.6...lost-found-v0.6.0) (2026-09-12)


### Features

* **catalog:** mark maturity and marketplace order ([9b9a3d0](https://github.com/PortakiApp/portaki-modules/commit/9b9a3d069d65151d266187094cf888ce0008dfc9))
* describe operation arguments with #[params] ([86f0fca](https://github.com/PortakiApp/portaki-modules/commit/86f0fcacc7ad3d17f370e819d0d6c045f51b95f4))
* **email:** localize module guest emails ([d7186a0](https://github.com/PortakiApp/portaki-modules/commit/d7186a0b21941d476d12d9a6c14e3965f5c4d057))
* **guest:** add module teasers and align nav labels ([860fbbd](https://github.com/PortakiApp/portaki-modules/commit/860fbbd188571d22247688849f8a2975fcf84cf0))
* **guest:** move report forms to overlay sheets ([47a31f7](https://github.com/PortakiApp/portaki-modules/commit/47a31f7afa1d032f85a1e988b1b296d6c3ae95d8))
* **lost-found:** add guest lost/found reports ([1ccc74f](https://github.com/PortakiApp/portaki-modules/commit/1ccc74f38eaf4258e912e28080f2ed32b8a5d5bd))
* **lost-found:** align host SDUI with design ([285d639](https://github.com/PortakiApp/portaki-modules/commit/285d6395618b8a40bfc089f0d1105852feb6d7cb))
* **lost-found:** host found, status, email context ([ba554d6](https://github.com/PortakiApp/portaki-modules/commit/ba554d6498c1ecbc8f4b70dd89781aff6f097ff4))
* **lost-found:** stay-action create modal surface ([e592d22](https://github.com/PortakiApp/portaki-modules/commit/e592d224269cbfb223d3bb98936a2ce870cdd81e))
* **lost-found:** TipTap create, hide empty stay ([9957d3c](https://github.com/PortakiApp/portaki-modules/commit/9957d3cd277577dfa32cca33d07ea5e2fa0fefbd))
* **modules:** déclarer les permissions des vingt et un modules ([b4f9e89](https://github.com/PortakiApp/portaki-modules/commit/b4f9e8902e6ed016f79a71c123f87da4c0b6d697))
* **modules:** opt into timed email catch-up ([8f826f4](https://github.com/PortakiApp/portaki-modules/commit/8f826f4198b6fc6b642045751f50cea3f2633ad4))
* **modules:** own mail and lost-found host SDUI ([a62e5be](https://github.com/PortakiApp/portaki-modules/commit/a62e5be10fa3049690cdb6185975d7175933e83e))
* **modules:** polish stay host SDUI cards ([3fdfef3](https://github.com/PortakiApp/portaki-modules/commit/3fdfef3785c7df15bdca95e978dc9e61cc4408ea))
* **modules:** Portaki author and sheet drawer hosts ([7366277](https://github.com/PortakiApp/portaki-modules/commit/7366277942fa71574f9059d40d43cd75536e79aa))


### Bug Fixes

* **clippy:** use sort_by_key for newest-first ([873eebb](https://github.com/PortakiApp/portaki-modules/commit/873eebb544d485e5f7107a7df1441db225b07c85))
* **modules:** bump lost-found and pre-arrival pins ([6e2d26d](https://github.com/PortakiApp/portaki-modules/commit/6e2d26dfadbaab7e358420e8ce6f5fe11dccfdf4))
* **stay-detail:** empty state when no reports ([da86cff](https://github.com/PortakiApp/portaki-modules/commit/da86cffadc4c0f4203d4864190af8cbc8a59aee9))

## [0.5.2]

### Added

- Manifest `emails[]` for `checkout-j2` → `sendCheckoutFollowUp` (tick-driven;
  no `catchUpOnPropertyPublish` — offset is fixed, module has no timing config).

## [0.3.1]

### Changed

- Host `create` description field uses TipTap `RichTextEditor` (not plain `TextArea`).
- Host `stay` hides when empty (no « Aucun objet déclaré… »); non-empty stay shows a
  Card with the declared-objects list and status controls.

## [0.3.0]

### Added

- Host surface `create` declared as `stay-action` — dashboard shows « Déclarer un
  objet trouvé » as a stay action button that opens a modal (form body only).
- Create form submits via `submitFound`; modal shell owns title / dismiss chrome.

### Changed

- Host `stay` is list/status only (no always-visible create form).

## [0.2.1]

### Fixed

- Bump semver so property pins can re-install a digest that includes `render_host_stay`
  (same-tag republish of `0.2.0` left installs on the old digest).

### Changed

- Host SDUI aligned with design `lostfound-editor-v1` / `foundObjectModal`:
  - `main` — info banner, TipTap guest note card, recent list with status pills (no create form).
  - `stay` — modal header (`guestName` · `stayDates`), TextArea description, FieldHint, « Envoyer au voyageur ».
  - Status labels: À récupérer / Envoyé / Récupéré; create always `to_collect`.

### Added

- Host `submitFound` command (multi-stay) + `lost-found.host-found` event for guest email.
- Report `status` (`to_collect` | `sent` | `returned`, default `to_collect`).
- Host `updateStatus` command + status Select on recent list rows.
- TipTap-ready descriptions / host note; `listForStay` accepts host `stayId`.
- `emailContext` returns `lostItemDescription` / `hasDeclaration` when stay reports exist (J+2 gate).

## [0.1.0]

### Added

- Initial `lost-found` module: guest form, host tip + recent list, `lost-found.submitted` event, `emailContext` (`checkoutTips`).
