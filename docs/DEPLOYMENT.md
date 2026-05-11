# Déploiement des packages npm

Le dépôt utilise **pnpm**. Tu peux committer un **`pnpm-lock.yaml`** après un `pnpm install` réussi (avec PAT GitHub `read:packages` pour GPR). Sans jeton en local, l’installation échoue tant que le SDK n’est pas résolu.

Les paquets utilisent le scope **`@portakiapp`** (minuscules), conforme à **GitHub Packages** pour l’organisation GitHub **PortakiApp**.

---

## Rôle de ce dépôt vs `portaki-sdk`

| Paquet | Où il est publié sur GPR |
|--------|-------------------------|
| **`@portakiapp/module-sdk`** | **[PortakiApp/portaki-sdk](https://github.com/PortakiApp/portaki-sdk)** uniquement |
| **`@portakiapp/module-train`**, **`-events`**, … | **Ce dépôt** (`portaki-modules`) |

Republier **`@portakiapp/module-sdk`** depuis deux dépôts différents vers GitHub Packages provoque en général **`403 Forbidden` / `write_package`** : le nom de paquet est déjà attaché au dépôt source du SDK.

Les modules sous **`modules/`** déclarent **`@portakiapp/module-sdk`** comme dépendance semver (voir `.npmrc` à la racine pour le registre GitHub Packages).

---

## GitHub Packages — workflow « Publish modules »

Fichier : `.github/workflows/publish-github-packages.yml`.

Comportement calqué sur **`publish-npm.yml`** de **portaki-sdk** :

1. `pnpm install`
2. Bump de version (push `develop` → préversion `…-develop.<run_number>`, release, ou `workflow_dispatch`)
3. `pnpm install --no-frozen-lockfile`
4. `pnpm publish` pour chaque module métier uniquement (**pas** `module-sdk`)

Authentification : **`NODE_AUTH_TOKEN`** (`GITHUB_TOKEN`) sur tout le job — nécessaire aussi pour **`pnpm install`** (téléchargement du SDK depuis GPR).

### Permissions GitHub

**Settings → Actions → General → Workflow permissions** : **Read and write**. Sinon `GITHUB_TOKEN` ne peut pas écrire dans Packages même avec `permissions: packages: write` dans le YAML.

### Consommer les paquets

`.npmrc` (adapter l’owner si besoin) :

```ini
@portakiapp:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${NODE_AUTH_TOKEN}
```

Installer par exemple :

```bash
pnpm add @portakiapp/module-sdk @portakiapp/module-train
```

(`module-sdk` résout vers l’artefact publié depuis **portaki-sdk**.)

---

## npmjs.com (optionnel)

Workflow manuel : **Publish packages (npmjs)** — secret **`NPM_TOKEN`**.

Packages proposés : **`@portakiapp/module-*`** métier uniquement (pas le SDK depuis ce repo).

---

## CI

`.github/workflows/ci.yml` : `pnpm install` + `pnpm lint`.

---

## Backend Java (`modules/pre-arrival-form/backend`)

Workflow : [`.github/workflows/publish-maven-github-packages.yml`](../.github/workflows/publish-maven-github-packages.yml) — `mvn deploy` vers **GitHub Packages Maven** (`https://maven.pkg.github.com/PortakiApp/portaki-modules`).

Déclenchement : push sur **`develop`** qui modifie `modules/pre-arrival-form/backend/`, ou **workflow_dispatch** manuel.

Dépendance **`app.portaki:portaki-module-sdk`** : résolue via le dépôt **`portaki-sdk`** sur GPR (`<url>https://maven.pkg.github.com/PortakiApp/portaki-sdk</url>` dans le `pom.xml`). Vérifie côté **portaki-sdk** que l’artefact Maven y est bien publié et que le package Maven autorise la lecture depuis **portaki-modules** (comme pour npm, *Manage Actions access* sur le package).
