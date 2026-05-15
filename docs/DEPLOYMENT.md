# Déploiement — npmjs & Maven Central

## npm — `@portaki/module-*`

- **Registre** : [registry.npmjs.org](https://www.npmjs.com/) (scope **`@portaki`**).
- **Automatique (semver)** : sur un **push `main`** du dépôt **`PortakiApp/portaki-modules`**, le workflow **[`ci.yml`](../.github/workflows/ci.yml)** (**Checks**) lance des **jobs matrix** : découverte des chemins via **`.github/scripts/discover-npm-publish-matrix.sh`**, puis pour chaque paquet **`@portaki/module-*`**, comparaison de version (Ruby `semver_local_gt_remote.rb`) et **`pnpm publish --filter …`** uniquement si la version locale est **strictement supérieure** à npmjs (Trusted Publishing OIDC, `id-token: write` sur les jobs de publication).
- **Trusted Publishing** : enregistrer le workflow **`ci.yml`** (job **Publish npm**) côté npmjs pour chaque paquet concerné (ou config équivalente org).
- **Manuel** : [`.github/workflows/publish-npm-packages.yml`](../.github/workflows/publish-npm-packages.yml) — **Publish npm**, `workflow_dispatch` (`all` ou un paquet). [`.github/workflows/modules-release-main.yml`](../.github/workflows/modules-release-main.yml) — **Module release** : uniquement `workflow_dispatch` (release GitHub `modules-v*` + `pnpm publish -r` version **unifiée** — flux historique / maintenance).
- **SDK** : **`@portaki/module-sdk`** est publié depuis [**portaki-sdk**](https://github.com/PortakiApp/portaki-sdk). Les modules ici déclarent **`"@portaki/module-sdk": "^…"`** (semver npm, pas de `file:`).

Script utilitaire : `node scripts/bump-workspace-versions.mjs ci-run <run>` pour aligner les patchs de build CI si besoin.

---

## Schéma JSON des manifestes

`pnpm validate:modules` télécharge **`schema/module.v1.json`** depuis la branche **main** de **portaki-sdk** (source de vérité du schéma).

---

## Maven — backends Java → **Maven Central**

- **Automatique (semver)** : sur un **push `main`** (`PortakiApp/portaki-modules`), le workflow **[`ci.yml`](../.github/workflows/ci.yml)** utilise **`.github/scripts/discover-maven-publish-matrix.sh`** (coordonnées lues depuis chaque `pom.xml` avec **`read-maven-pom-coords.py`**) puis des jobs matrix **`publish-maven-registry`** : chaque backend **release** (pas `-SNAPSHOT`) n’est déployé avec **`mvn deploy -P central-deploy`** que si la version du POM est **strictement supérieure** à `<latest>` dans `maven-metadata.xml` sur repo1.maven.org. Nécessite **`OSSRH_*`** + **GPG** ; sinon le job affiche une notice et ne déploie pas.
- **Manuel (tous les backends)** : [`.github/workflows/publish-maven-central.yml`](../.github/workflows/publish-maven-central.yml) — **Java to Central**, uniquement **`workflow_dispatch`** : déploie toute la matrice découverte (sans comparaison semver).

### Secrets (même jeu que **portaki-sdk**)

Configurer sur le dépôt **portaki-modules** (ou en secrets d’organisation) : **`OSSRH_USERNAME`**, **`OSSRH_TOKEN`**, **`GPG_PRIVATE_KEY`**, **`GPG_PASSPHRASE`** — voir [Central Portal](https://central.sonatype.org/publish/generate-portal-token/) et [exigences GPG](https://central.sonatype.org/publish/requirements/gpg/).

### Namespace Maven

Les coordonnées **`app.portaki.module:*`** (et tout autre `groupId` utilisé dans un `pom.xml` ici) doivent être **autorisées** sur votre compte Central Portal.

### Dépendance SDK

**`app.portaki:portaki-module-sdk`** : release sur **Maven Central** uniquement (plus de dépôt GPR pour ces JAR).

### Backend `ical-sync`

Artefact **`app.portaki.module:ical-sync-backend`** : logique iCal ; l’API **portaki-api** invoque ce module via **`HostModuleBackendRunPort`**.

---

## CI

- [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) : **Checks** (push `main`/`develop`, PR) + publication **semver** npm / Maven sur **`main`**.
- [`.github/workflows/modules-release-main.yml`](../.github/workflows/modules-release-main.yml) : **Module release** — `workflow_dispatch` uniquement (release GitHub + npm version unifiée).
- [`.github/workflows/publish-maven-central.yml`](../.github/workflows/publish-maven-central.yml) : **Java to Central** — `workflow_dispatch` uniquement (déploiement matriciel sans gate semver).
