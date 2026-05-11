# Déploiement des packages npm

Ce dépôt publie les modules sous le scope **`@portaki`** via **pnpm**. La CI installe avec `pnpm install --frozen-lockfile` : le fichier `pnpm-lock.yaml` doit être commité après tout changement de dépendances.

## Prérequis côté npmjs

1. Créer un compte sur [npmjs.com](https://www.npmjs.com/) et rejoindre ou créer l’organisation **`@portaki`** (packages scoped publics).
2. Générer un **token d’accès** avec permission **Publish** (classic token ou granular selon votre politique).
3. Dans le dépôt GitHub : **Settings → Secrets and variables → Actions** → ajouter **`NPM_TOKEN`** avec ce token.

Ordre recommandé lors d’une publication « all » : le workflow publie d’abord **`@portaki/module-sdk`**, puis les modules métier. Les dépendances `workspace:*` sont réécrites par pnpm vers les versions réellement publiées au moment du `pnpm publish`.

Avant la première release, incrémenter les champs **`version`** dans les `package.json` concernés (semver).

## Workflow « Publish packages (npmjs) »

1. Onglet **Actions** du dépôt GitHub.
2. Choisir **Publish packages (npmjs)** → **Run workflow**.
3. Sélectionner le package (ou **all**).
4. Vérifier les journaux : une version déjà publiée avec le même numéro fera échouer la tâche (comportement npm normal).

Fichier : `.github/workflows/publish-npm.yml`.

## GitHub Packages (registre npm intégré)

Workflow : **Publish packages (GitHub Packages)** (`.github/workflows/publish-github-packages.yml`).

Contrainte importante : sur GitHub Packages, le **scope npm** (`@portaki` dans le champ `name` du `package.json`) doit en général **correspondre au propriétaire GitHub** du dépôt ou à une convention documentée par GitHub pour votre organisation. Si votre organisation GitHub s’appelle par exemple `PortakiApp` et le scope npm `@portaki`, il peut être nécessaire soit de publier sur **npmjs** plutôt que sur GPR, soit d’aligner le naming (`@portakiapp/*`) selon votre stratégie.

Permissions du workflow : `packages: write` (déjà déclaré). Le secret **`GITHUB_TOKEN`** fourni par GitHub Actions suffit pour pousser des artefacts vers GitHub Packages du même dépôt / organisation, sous réserve des règles de scope ci-dessus.

Configurer le client npm / pnpm côté application consommatrice, par exemple :

```ini
@portaki:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

## CI continue

Le workflow **CI** (`.github/workflows/ci.yml`) ne publie rien : il valide que le workspace s’installe et que les scripts `lint` passent.

## Backend Java (`pre-arrival-form/backend`)

Ce dossier n’est **pas** publié par les workflows npm. Il suit le cycle de build Maven / déploiement de votre plateforme (image Docker, registry privé, etc.).
