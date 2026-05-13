# Déploiement — npmjs & Maven (GitHub Packages)

## npm — `@portaki/module-*`

- **Registre** : [registry.npmjs.org](https://www.npmjs.com/) (scope **`@portaki`**).
- **Push `main`** : workflow **[`modules-release-main.yml`](../.github/workflows/modules-release-main.yml)** — vérifie le workspace, **publie tous les `@portaki/module-*`**, puis crée une **release GitHub** `modules-vX.Y.Z` avec **notes auto** (PR mergées depuis la release précédente). Nécessite d’avoir **aligné la même `version`** dans chaque `modules/<id>/package.json` et `modules/pre-arrival-form/frontend/package.json` avant merge (script d’assert dans la CI).
- **Trusted Publishing** : ajouter le workflow **`modules-release-main.yml`** (fichier YAML) sur npmjs pour chaque paquet **`@portaki/module-*`** (ou configuration équivalente côté org), avec permission **`id-token: write`** dans le job (déjà présent).
- **Manuel** : [`.github/workflows/publish-npm.yml`](../.github/workflows/publish-npm.yml) — **`workflow_dispatch`**, choix **`all`** ou un paquet précis.
- **SDK** : **`@portaki/module-sdk`** est publié depuis [**portaki-sdk**](https://github.com/PortakiApp/portaki-sdk) (`publish-npm-sdk.yml` et **`sdk-release-main.yml`** sur push `main`). Les modules ici déclarent **`"@portaki/module-sdk": "^…"`** (semver npm, pas de `file:`).

Script utilitaire : `node scripts/bump-workspace-versions.mjs ci-run <run>` pour aligner les patchs de build CI si besoin.

---

## Schéma JSON des manifestes

`pnpm validate:modules` télécharge **`schema/module.v1.json`** depuis la branche **main** de **portaki-sdk** (source de vérité du schéma).

---

## Maven — backend `pre-arrival-form`

Workflow : [`.github/workflows/publish-maven-github-packages.yml`](../.github/workflows/publish-maven-github-packages.yml) — `mvn deploy` vers **GitHub Packages Maven**.

Déclenchement : push sur **`main`** qui modifie `modules/pre-arrival-form/backend/`, ou **workflow_dispatch**.

Dépendance **`app.portaki:portaki-module-sdk`** : version **release** attendue sur **Maven Central** (la CI et la publication GPR n’installent plus le SDK depuis un clone). L’action **`maven-gpr-install-java-sdk`** du dépôt **portaki-sdk** ne fait que JDK + `settings.xml` GPR ; l’option `install-sdk-from-source: true` + `checkout-portaki-sdk` reste possible pour un scénario hors Central.

---

## CI

- [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) : job unique **`checks`** avec **`strategy.matrix`** (`node-workspace`, `java-backend`) — en parallèle, `fail-fast: false`. Réutilise **`pnpm-workspace-setup`** et **`maven-gpr-install-java-sdk`** depuis [**portaki-sdk** `/.github/actions`](https://github.com/PortakiApp/portaki-sdk/tree/main/.github/actions) (`@main` ou tag). Le volet **Java** est ignoré sur les PR depuis un fork (pas d’accès GPR). Noms de checks GitHub : **`checks (node-workspace)`**, **`checks (java-backend)`** (à référencer dans les règles de branche si besoin).
- [`.github/workflows/modules-release-main.yml`](../.github/workflows/modules-release-main.yml) : sur **push `main`** (chemins `modules/**`, lockfile, etc.), **vérifie** puis **publie npm** + **release GitHub** `modules-v*` si la version unifiée n’a pas encore été releasée.
