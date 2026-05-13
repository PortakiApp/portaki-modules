# Déploiement — npmjs & Maven (GitHub Packages)

## npm — `@portaki/module-*`

- **Registre** : [registry.npmjs.org](https://www.npmjs.com/) (scope **`@portaki`**).
- **Push `main`** : workflow **[`modules-release-main.yml`](../.github/workflows/modules-release-main.yml)** (**Module release** dans l’onglet Actions) — vérifie le workspace, **publie tous les `@portaki/module-*`**, puis crée une **release GitHub** `modules-vX.Y.Z` avec **notes auto** (PR mergées depuis la release précédente). Nécessite d’avoir **aligné la même `version`** dans chaque `modules/<id>/package.json` et `modules/pre-arrival-form/frontend/package.json` avant merge (script d’assert dans la CI).
- **Trusted Publishing** : ajouter le workflow **`modules-release-main.yml`** (fichier YAML) sur npmjs pour chaque paquet **`@portaki/module-*`** (ou configuration équivalente côté org), avec permission **`id-token: write`** dans le job (déjà présent).
- **Manuel** : [`.github/workflows/publish-npm-packages.yml`](../.github/workflows/publish-npm-packages.yml) — dans Actions, workflow **Publish npm** ; **`workflow_dispatch`**, choix **`all`** ou un paquet précis. Si vous utilisez **Trusted Publishing** npm, mettre à jour le fichier de workflow côté npmjs (**`publish-npm-packages.yml`**, plus **`publish-npm.yml`**).
- **SDK** : **`@portaki/module-sdk`** est publié depuis [**portaki-sdk**](https://github.com/PortakiApp/portaki-sdk) (`publish-npm-sdk.yml` et **`sdk-release-main.yml`** sur push `main`). Les modules ici déclarent **`"@portaki/module-sdk": "^…"`** (semver npm, pas de `file:`).

Script utilitaire : `node scripts/bump-workspace-versions.mjs ci-run <run>` pour aligner les patchs de build CI si besoin.

---

## Schéma JSON des manifestes

`pnpm validate:modules` télécharge **`schema/module.v1.json`** depuis la branche **main** de **portaki-sdk** (source de vérité du schéma).

---

## Maven — backends Java → **Maven Central**

Workflow : [`.github/workflows/publish-maven-central.yml`](../.github/workflows/publish-maven-central.yml) — dans Actions : **Java to Central**. **`mvn deploy -P central-deploy`** pour **`pre-arrival-form-backend`** puis **`ical-sync-backend`** (versions **release** sans `-SNAPSHOT`).

Déclenchement : push sur **`main`** qui modifie un `modules/*/backend/**`, ou **`workflow_dispatch`**.

### Secrets (même jeu que **portaki-sdk**)

Configurer sur le dépôt **portaki-modules** (ou en secrets d’organisation) : **`OSSRH_USERNAME`**, **`OSSRH_TOKEN`**, **`GPG_PRIVATE_KEY`**, **`GPG_PASSPHRASE`** — voir [Central Portal](https://central.sonatype.org/publish/generate-portal-token/) et [exigences GPG](https://central.sonatype.org/publish/requirements/gpg/).

### Namespace Maven

Les coordonnées **`app.portaki.module:*`** doivent être **autorisées** sur votre compte Central Portal (ajout du namespace si besoin, comme pour **`app.portaki`**).

### Dépendance SDK

**`app.portaki:portaki-module-sdk`** : release sur **Maven Central** uniquement (plus de dépôt GPR pour ces JAR).

### Backend `ical-sync`

Artefact **`app.portaki.module:ical-sync-backend`** : logique iCal (fetch HTTPS sécurisé, parsing, mise à jour `last_sync_at` / `sync_summary`) ; l’API **portaki-api** ne contient plus de code iCal dédié et invoque ce module via le port **`HostModuleBackendRunPort`**.

---

## CI

- [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) : workflow **Checks** — matrice **`node-workspace`**, **`java — pre-arrival-form`**, **`java — ical-sync`** (`fail-fast: false`). Réutilise **`pnpm-workspace-setup`** et **`maven-gpr-install-java-sdk`** depuis [**portaki-sdk** `/.github/actions`](https://github.com/PortakiApp/portaki-sdk/tree/main/.github/actions). Les jobs Java sont ignorés sur les PR depuis un fork. Mettre à jour les **statuts requis** des branches si besoin (`Checks (java — pre-arrival-form)`, etc.).
- [`.github/workflows/modules-release-main.yml`](../.github/workflows/modules-release-main.yml) : workflow **Module release** — sur **push `main`** (chemins `modules/**`, lockfile, etc.), **vérifie** puis **publie npm** + **release GitHub** `modules-v*` si la version unifiée n’a pas encore été releasée.
