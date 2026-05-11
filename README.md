# Portaki Modules

[![CI](https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Monorepo **pnpm** des modules UI guest Portaki : chaque dossier publie un package npm `@portaki/module-*` construit avec [`@portaki/module-sdk`](./packages/module-sdk). Les écrans sont des modules React consommés par l’application guest ; le backend Java vit à part (`pre-arrival-form/backend/`).

## Sommaire

- [Démarrage rapide](#démarrage-rapide)
- [Structure du dépôt](#structure-du-dépôt)
- [Packages publiés](#packages-publiés)
- [Documentation](#documentation)
- [Publication & CI](#publication--ci)

## Démarrage rapide

Prérequis : **Node.js 22+**, **pnpm 9** (via Corepack ou `npx pnpm@9.15.4`).

```bash
pnpm install
pnpm lint
```

Chaque module expose un **export par défaut** : `definePortakiModule({ ... })` depuis `@portaki/module-sdk`.

## Structure du dépôt

```text
packages/module-sdk/     # Contrat TypeScript partagé (@portaki/module-sdk)
train/                   # Horaires trains (Navitia)
events/                  # Événements + hooks carte
rules/                   # Règlement intérieur
appliances/              # Guide appareils
checklist/               # Checklist départ
pre-arrival-form/
  frontend/              # Module formulaire avant arrivée
  backend/               # Consommation Axon / Spring (non npm)
```

## Packages publiés

| Package | Description |
|--------|-------------|
| [`@portaki/module-sdk`](./packages/module-sdk) | Types et `definePortakiModule` |
| [`@portaki/module-train`](./train) | Horaires SNCF / Navitia |
| [`@portaki/module-events`](./events) | Événements & carte |
| [`@portaki/module-rules`](./rules) | Règlement |
| [`@portaki/module-appliances`](./appliances) | Appareils |
| [`@portaki/module-checklist`](./checklist) | Checklist départ |
| [`@portaki/module-pre-arrival-form`](./pre-arrival-form/frontend) | Formulaire avant arrivée |

## Documentation

| Fichier | Contenu |
|---------|---------|
| [docs/README.md](./docs/README.md) | Index de la doc produit / technique |
| [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md) | Prérequis registry, secrets GitHub, procédure de release |
| [docs/MODULE_README_SCHEMA.md](./docs/MODULE_README_SCHEMA.md) | **Schéma unique** des README modules (réutilisable landing) |

Chaque sous-dossier de module contient un `README.md` qui suit ce schéma.

## Publication & CI

- **CI** : workflow [`.github/workflows/ci.yml`](./.github/workflows/ci.yml) — `pnpm install --frozen-lockfile` + `pnpm lint` sur `main` / PRs.
- **npmjs** : [`.github/workflows/publish-npm.yml`](./.github/workflows/publish-npm.yml) — déclenchement manuel ; secret `NPM_TOKEN`.
- **GitHub Packages** : [`.github/workflows/publish-github-packages.yml`](./.github/workflows/publish-github-packages.yml) — déclenchement manuel ; token `GITHUB_TOKEN` (permissions `packages:write`). Voir les contraintes de scope dans [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md).

Licence : **AGPL-3.0** (cf. chaque `package.json`).
