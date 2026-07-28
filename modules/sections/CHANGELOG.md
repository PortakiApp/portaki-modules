# Changelog

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
