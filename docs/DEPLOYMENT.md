# Déploiement — npmjs & Maven (GitHub Packages)

## npm — `@portaki/module-*`

- **Registre** : [registry.npmjs.org](https://www.npmjs.com/) (scope **`@portaki`**).
- **Workflow** : [`.github/workflows/publish-npm.yml`](../.github/workflows/publish-npm.yml) — **`workflow_dispatch`**, choix **`all`** ou un paquet précis.
- **Trusted Publishing** : configurer sur npmjs pour chaque paquet : org **PortakiApp**, dépôt **portaki-modules**, workflow **`publish-npm.yml`** (fichier YAML), permission **`id-token: write`** dans le job (déjà présent).
- **Version** : bump manuel des **`version`** dans chaque `modules/<nom>/package.json` (et `pre-arrival-form/frontend` si concerné) avant publication.
- **SDK** : **`@portaki/module-sdk`** est publié uniquement depuis [**portaki-sdk**](https://github.com/PortakiApp/portaki-sdk) (`publish-npm-sdk.yml`). Les modules ici déclarent **`"@portaki/module-sdk": "^…"`** (semver npm, pas de `file:`).

Script utilitaire : `node scripts/bump-workspace-versions.mjs ci-run <run>` pour aligner les patchs de build CI si besoin.

---

## Schéma JSON des manifestes

`pnpm validate:modules` télécharge **`schema/module.v1.json`** depuis la branche **main** de **portaki-sdk** (source de vérité du schéma).

---

## Maven — backend `pre-arrival-form`

Workflow : [`.github/workflows/publish-maven-github-packages.yml`](../.github/workflows/publish-maven-github-packages.yml) — `mvn deploy` vers **GitHub Packages Maven**.

Déclenchement : push sur **`main`** qui modifie `modules/pre-arrival-form/backend/`, ou **workflow_dispatch**.

Dépendance **`app.portaki:portaki-module-sdk`** : résolue via GPR / clone **portaki-sdk** (voir script CI du workflow).

---

## CI

[`.github/workflows/ci.yml`](../.github/workflows/ci.yml) : `pnpm install --frozen-lockfile`, `pnpm assert-no-file-deps`, `pnpm validate:modules`, `pnpm lint`, job Maven sur le backend pré-arrivée.
