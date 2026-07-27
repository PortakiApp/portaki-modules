# Marketplace catalogue order

Host marketplace and registry list sort by `maturity` then `sortOrder` (lower first).
Declared in each module's `portaki.module.json`.

## Stable (shown first)

High-value guest booklet modules that should work end-to-end.

| sortOrder | id | Notes |
|---:|---|---|
| 10 | `wifi-guest` | |
| 20 | `access-guide` | |
| 30 | `rules` | |
| 40 | `sections` | |
| 50 | `pre-arrival-form` | |
| 60 | `checklist` | |
| 70 | `emergency-contacts` | |
| 80 | `lost-found` | |
| 90 | `issue-report` | |
| 100 | `guest-reviews` | |
| 110 | `consumables` | |
| 120 | `appliances` | |
| 130 | `weather` | |
| 140 | `local-guide` | |
| 150 | `facility-hours` | |
| 160 | `waste-recycling` | |
| 170 | `ev-parking` | |

## Beta (shown after, with Beta badge)

Product choice (2026-07): incomplete, experimental, or niche integrations.

| sortOrder | id | Why beta |
|---:|---|---|
| 200 | `train` | Static mock TER board — no Navitia / host station config yet |
| 210 | `events` | OpenAgenda nearby fetch is experimental; needs keys / radius setup |
| 220 | `ical-sync` | Host calendar import still early (v0.3.x, beta) |
| 230 | `nuki` | Smart-lock provider; hardware + BYOK; guest UX lives in access-guide |

Promote a module out of beta by setting `maturity: "stable"` and a sortOrder
in the stable band, then bump the module semver (manifest is published to OCI).
