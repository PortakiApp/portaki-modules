# Changelog

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
