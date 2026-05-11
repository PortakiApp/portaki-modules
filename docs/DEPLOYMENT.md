# Déploiement des packages npm

Le dépôt utilise **pnpm** (`pnpm-lock.yaml` à committer). En local : `pnpm install --frozen-lockfile`.

Les paquets utilisent le scope **`@portakiapp`** (minuscules), conforme à **GitHub Packages** pour l’organisation GitHub **PortakiApp**.

---

## Rôle de ce dépôt vs `portaki-sdk`

| Paquet | Où il est publié sur GPR |
|--------|-------------------------|
| **`@portakiapp/module-sdk`** | **[PortakiApp/portaki-sdk](https://github.com/PortakiApp/portaki-sdk)** uniquement |
| **`@portakiapp/module-train`**, **`-events`**, … | **Ce dépôt** (`portaki-modules`) |

Republier **`@portakiapp/module-sdk`** depuis deux dépôts différents vers GitHub Packages provoque en général **`403 Forbidden` / `write_package`** : le nom de paquet est déjà attaché au dépôt source du SDK.

Le dossier **`packages/module-sdk`** ici sert de **copie workspace** pour le développement local (`workspace:*`) ; il n’est **pas** publié par `.github/workflows/publish-github-packages.yml`.

---

## GitHub Packages — workflow « Publish modules »

Fichier : `.github/workflows/publish-github-packages.yml`.

Comportement calqué sur **`publish-npm.yml`** de **portaki-sdk** :

1. `pnpm install --frozen-lockfile`
2. Bump de version (push `develop` → préversion `…-develop.<run_number>`, release, ou `workflow_dispatch`)
3. `pnpm install --no-frozen-lockfile`
4. `pnpm publish` pour chaque module métier uniquement (**pas** `module-sdk`)

Authentification : **`NODE_AUTH_TOKEN: ${{ secrets.GITHUB_TOKEN }}`** sur l’étape de publication.

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

`.github/workflows/ci.yml` : `pnpm install --frozen-lockfile` + `pnpm lint`.

---

## Backend Java (`pre-arrival-form/backend`)

Non couvert par les workflows npm ci-dessus.
