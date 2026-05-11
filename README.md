# Portaki Modules

[![CI](https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

Monorepo **pnpm** des modules UI guest Portaki : packages sous le scope **`@portakiapp`** (règle GitHub Packages pour l’org **PortakiApp**). Le SDK **`@portakiapp/module-sdk`** est publié depuis **[portaki-sdk](https://github.com/PortakiApp/portaki-sdk)** ; ce dépôt contient une copie workspace pour le dev local et **ne republie pas** le SDK sur GPR. Les écrans sont des modules React pour l’app guest ; le backend Java vit à part (`pre-arrival-form/backend/`).

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

Chaque module expose un **export par défaut** : `definePortakiModule({ ... })` depuis `@portakiapp/module-sdk`.

## Structure du dépôt

```text
packages/module-sdk/     # Copie locale du contrat (même nom que le paquet GPR publié depuis portaki-sdk)
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
| `@portakiapp/module-sdk` | Publié depuis **portaki-sdk** ; copie locale dans `./packages/module-sdk` |
| [`@portakiapp/module-train`](./train) | Horaires SNCF / Navitia |
| [`@portakiapp/module-events`](./events) | Événements & carte |
| [`@portakiapp/module-rules`](./rules) | Règlement |
| [`@portakiapp/module-appliances`](./appliances) | Appareils |
| [`@portakiapp/module-checklist`](./checklist) | Checklist départ |
| [`@portakiapp/module-pre-arrival-form`](./pre-arrival-form/frontend) | Formulaire avant arrivée |

## Documentation

| Fichier | Contenu |
|---------|---------|
| [docs/README.md](./docs/README.md) | Index de la doc produit / technique |
| [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md) | Prérequis registry, secrets GitHub, procédure de release |
| [docs/MODULE_README_SCHEMA.md](./docs/MODULE_README_SCHEMA.md) | **Schéma unique** des README modules (réutilisable landing) |

Chaque sous-dossier de module contient un `README.md` qui suit ce schéma.

## Publication & CI

- **CI** : workflow [`.github/workflows/ci.yml`](./.github/workflows/ci.yml) — `pnpm install --frozen-lockfile` + `pnpm lint` sur `develop` / `main` / PRs.
- **GitHub Packages** (comme [portaki-sdk](https://github.com/PortakiApp/portaki-sdk)) : [`.github/workflows/publish-github-packages.yml`](./.github/workflows/publish-github-packages.yml) — push sur **`develop`**, **release** ou **workflow_dispatch** ; publie **`@portakiapp/module-*`** **sauf** `module-sdk` (déjà fourni par portaki-sdk). Détail dans [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md).
- **npmjs** : [`.github/workflows/publish-npm.yml`](./.github/workflows/publish-npm.yml) — déclenchement manuel ; secret **`NPM_TOKEN`** ; packages **`@portakiapp/module-*`** (hors SDK).

Licence : **AGPL-3.0** (cf. chaque `package.json`).
