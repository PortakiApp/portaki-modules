# Déploiement des packages npm

Le dépôt utilise **pnpm** (`pnpm-lock.yaml` à committer). En local : `pnpm install --frozen-lockfile`.

Les modules peuvent être publiés soit sur **GitHub Packages** (flux principal, aligné sur [PortakiApp/portaki-sdk](https://github.com/PortakiApp/portaki-sdk)), soit sur **npmjs.com** (workflow manuel optionnel).

---

## GitHub Packages (recommandé — comme `portaki-sdk`)

Le workflow **Publish modules (GitHub Packages)** (`.github/workflows/publish-github-packages.yml`) reprend la même logique que **Publish JavaScript (GitHub Packages)** dans `portaki-sdk` :

- Authentification avec **`GITHUB_TOKEN`** uniquement (`permissions: packages: write`).
- Le scope npm publié est **`@<propriétaire_github_en_minuscules>/…`** (ex. `@portakiapp/module-sdk` pour l’organisation **PortakiApp**). Le job exécute `scripts/align-npm-scope-for-gpr.mjs` pour réécrire `@portaki/*` → `@owner/*` **uniquement dans le runner CI** (les fichiers du dépôt restent en `@portaki/*` pour le dev local).
- Ordre de publication : **`module-sdk`** puis les modules métier.

### Déclencheurs

| Événement | Version publiée |
|-----------|-----------------|
| **Push sur `develop`** (chemins filtrés : code des packages, `scripts/`, ce workflow) | `<base>-develop.<run_number>` (ex. `0.1.0-develop.42`) pour **tous** les packages — une nouvelle version à chaque push afin d’éviter les collisions npm. |
| **Release GitHub** publiée | Tag parsé : préfixes acceptés `modules-v` ou `v` (ex. `modules-v0.2.0` → `0.2.0`). |
| **workflow_dispatch** | Champ optionnel **version** : si renseigné, cette semver est appliquée à tous les `package.json` publishables ; si vide, les versions du dépôt sont utilisées telles quelles (échec si déjà publiée). |

### Consommer les paquets (app ou CI)

Comme pour `portaki-sdk`, ajoutez un `.npmrc` :

```ini
@portakiapp:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

Remplace `portakiapp` par **ton owner GitHub en minuscules**. Installe ensuite par exemple :

```bash
pnpm add @portakiapp/module-sdk @portakiapp/module-train
```

Pour un PAT lecture : `read:packages`. Dans une GitHub Action du **même** dépôt, `GITHUB_TOKEN` peut suffire selon les permissions du workflow.

---

## npmjs.com (optionnel)

### Prérequis

1. Organisation **`@portaki`** (ou autre scope autorisé) sur [npmjs.com](https://www.npmjs.com/).
2. Secret **`NPM_TOKEN`** dans les GitHub Actions du dépôt.

### Workflow « Publish packages (npmjs) »

Déclenchement manuel uniquement : **Publish packages (npmjs)** → choix du package ou **all**.

Fichier : `.github/workflows/publish-npm.yml`.

Les noms restent **`@portaki/module-*`** sur ce registre (pas d’alignement automatique du propriétaire GitHub).

---

## CI continue

Le workflow **CI** (`.github/workflows/ci.yml`) ne publie pas : `pnpm install --frozen-lockfile` + `pnpm lint`.

---

## Backend Java (`pre-arrival-form/backend`)

Non publié par ces workflows npm. Suivre votre pipeline Maven / Docker habituel.
