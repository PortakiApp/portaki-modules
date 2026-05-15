<p align="center">
  <a href="https://www.portaki.app" title="Portaki">
    <img src="https://www.portaki.app/icon.svg" width="88" height="88" alt="Portaki">
  </a>
</p>

<h1 align="center">Portaki Modules</h1>

<p align="center">
  Monorepo <strong>pnpm</strong> des modules UI invités officiels — scope npm <strong><code>@portaki/module-*</code></strong><br/>
  <sub>Publication sur <a href="https://www.npmjs.com/org/portaki">npmjs</a> · SDK JS dans <a href="https://github.com/PortakiApp/portaki-sdk">portaki-sdk</a></sub>
</p>

<p align="center">
  <a href="https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml"><img src="https://github.com/PortakiApp/portaki-modules/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://www.gnu.org/licenses/agpl-3.0"><img src="https://img.shields.io/badge/License-AGPL%20v3-blue.svg" alt="AGPL-3.0"></a>
  <a href="https://github.com/PortakiApp/portaki-sdk"><img src="https://img.shields.io/badge/SDK-portaki--sdk-181717?logo=github" alt="SDK"></a>
</p>

---

## Rôle de ce dépôt

Les sources des paquets **`@portaki/module-*`** (invité) vivent sous **`modules/`**. Chaque module déclare **`@portaki/module-sdk": "^x.y.z"`** depuis le registre public (pas de `workspace:` ni `file:`). Le manifeste **`portaki.module.json`** est lu par l’API (**portaki-api**) via GitHub Contents sur ce dépôt, dossier **`modules/`**.

Le SDK **`@portaki/module-sdk`** (types, `definePortakiModule`) est publié depuis **[portaki-sdk](https://github.com/PortakiApp/portaki-sdk)** uniquement.

## Aperçus documentation

Les README sous `modules/*/README.md` embarquent une **illustration factice** (SVG type capture mobile) référencée depuis le dépôt **portaki-web** (`../portaki-web/public/module-previews/`). Régénération des visuels : `cd ../portaki-web && pnpm run generate:module-previews`. Réinjection des sections « Aperçu » dans les README : `node scripts/inject-readme-previews.mjs`.

---

## Démarrage rapide

**Prérequis** : Node.js 22+, pnpm 9.

```bash
pnpm install
pnpm validate:modules
pnpm lint
```

---

## Structure

```text
modules/
  train/                 # Horaires trains (Navitia)
  events/                # Événements + carte
  …                      # Voir modules/ pour la liste complète
  pre-arrival-form/
    frontend/            # paquet npm @portaki/module-pre-arrival-form
    backend/             # Java / Axon (Maven)
```

---

## Packages publiés (npmjs)

| Package | Description |
|---------|-------------|
| `@portaki/module-sdk` | Publié depuis [**portaki-sdk**](https://github.com/PortakiApp/portaki-sdk) |
| `@portaki/module-train` | Horaires SNCF / Navitia |
| `@portaki/module-events` | Événements & carte |
| `@portaki/module-rules` | Règlement |
| `@portaki/module-appliances` | Appareils |
| `@portaki/module-checklist` | Checklist départ |
| `@portaki/module-pre-arrival-form` | Formulaire avant arrivée |
| … | Tout dossier sous `modules/` avec `package.json` + `portaki.module.json` |

---

## Documentation

| Fichier | Contenu |
|---------|---------|
| [docs/README.md](./docs/README.md) | Index doc |
| [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md) | npmjs, Trusted Publishing, Maven |
| [docs/MODULE_README_SCHEMA.md](./docs/MODULE_README_SCHEMA.md) | Schéma README modules |

---

## CI & publication

- **Checks** : [`.github/workflows/ci.yml`](./.github/workflows/ci.yml) — lint, validation des manifestes, `mvn verify` sur **tous** les `modules/*/backend` découverts par script, puis sur un **push `main`** (dépôt `PortakiApp/portaki-modules` uniquement) :
  - **npm** : `pnpm registry-publish:npm` — chaque `@portaki/module-*` est publié seulement si `package.json#version` est **strictement supérieure** (semver) à la dernière version sur npmjs.
  - **Maven** : `pnpm registry-publish:maven` — chaque backend est déployé sur Central seulement si la version du `pom.xml` est **strictement supérieure** à `<latest>` dans `maven-metadata.xml` (secrets OSSRH + GPG requis ; sinon le job skip avec une notice).
- **Manuel** : [`.github/workflows/publish-npm-packages.yml`](./.github/workflows/publish-npm-packages.yml) (npm ciblé), [`.github/workflows/publish-maven-central.yml`](./.github/workflows/publish-maven-central.yml) (tous les JAR sans gate semver), [`.github/workflows/modules-release-main.yml`](./.github/workflows/modules-release-main.yml) (release GitHub + npm version unifiée historique).

Détail : [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md).

---

## Licence

**AGPL-3.0** — cf. chaque `package.json`.
