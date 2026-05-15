# Mémo architecture modules & plateforme (Wasm, scale, ops)

**Objectif** : reprendre dans 6 mois (ou plus) la discussion d’architecture sans tout redérouler.  
**Date de rédaction** : mai 2026.  
**Périmètre** : modules invités / hôte, scalabilité catalogue, exécution sandbox, données, coûts, ce qui est déjà en place vs chantier futur.

**Documents liés** (plus courts, orientés implémentation actuelle) :

- [`MODULE_PLATFORM_PREPARATION.md`](./MODULE_PLATFORM_PREPARATION.md) — checklist préparation + lien code.
- Journal multi-dépôts : si tu utilises un **`AGENTS.md`** à la racine du workspace Cursor (**Repositories/**), y recopier les décisions datées ; ce mémo est la **source longue** versionnée ici.

---

## 1. Contexte métier

- Portaki : **API Java** (Spring, hexagonal), **Axon + RabbitMQ** sur certains domaines, **modules** sous forme de paquets **`@portaki/module-*`** (npm) + backends Java optionnels, manifestes **`portaki.module.json`**, dépôt **`portaki-modules`**, SDK **`@portaki/module-sdk`** / schéma dans **`portaki-sdk`**.
- Ambition : **beaucoup de modules** (centaines / 1000+), **sans** explosion de microservices ni **rebuild systématique** de l’API à chaque bump de module.

---

## 2. Piste initialement évoquée (et son verdict)

**Idée** : modules en **JavaScript**, déployés comme **fonctions type Lambda**, avec **setup** qui crée tables / SQL, mises à jour pareil — hypothèse « plus léger que la JVM ».

**Ce qui est pertinent** :

- **Découplage** release module vs release core.
- **IaC / migrations versionnées** packagées avec le module (bon principe).

**Ce qui pose problème si on reste sur « Lambda + DDL live »** :

- Mélanger **déploiement de code** et **mutation de schéma** au runtime = audit, rollback et review difficiles.
- **Multi-tenant** + DDL ad hoc = risque opérationnel et sécurité (privilèges DB).
- **« Plus léger que Java »** n’est pas automatique : cold starts, timeouts, VPC/DB, observabilité × N fonctions, facturation à fort volume.

**Synthèse** : garder l’**intuition** (artefacts indépendants, schéma possédé par le module, pas de monolithe Maven pour chaque module), mais **séparer** exécution métier et **application** des migrations (job contrôlé, pas DDL dans la fonction).

---

## 3. Direction d’architecture recommandée (cible long terme)

### 3.1 Artefact module

- **WebAssembly** (ex. **Extism** ou **Wasmtime** / écosystème type **Spin**, **WasmCloud**) : binaire **`.wasm`** versionné, **polyglotte** (TS, Rust, Go…), **sandbox** par défaut.
- **Registre** type **OCI / GHCR** pour les `.wasm` (+ métadonnées), **pas** uniquement une dépendance Maven dans `portaki-api` pour le runtime module.

### 3.2 Où tourne le Wasm ?

| Phase / charge | Recommandation |
|----------------|----------------|
| Peu de modules, peu de versions | **In-process** dans l’hôte possible (latence µs, simplicité). |
| **1000+ modules × versions × tenants** | **Service dédié** `portaki-module-runtime` (plusieurs pods), **cache LRU** des instances hot, **hash consistent** ou partitionnement pour répartir la RAM. |
| Règle module | **Stateless** entre invocations : état en **DB** (schéma module) ou **cache** ; sinon ce n’est plus un « module » mais un service à part. |

**Pourquoi un runtime séparé à grande échelle** : éviter la **saturation RAM** de la JVM si on chargeait tout in-process (ordre de grandeur discuté : mémoire par instance × nombre de combinaisons).

### 3.3 Données (DB)

- **Base « modules » séparée du core** métier (blast radius, backups).
- **Un schéma Postgres par module** (ou équivalent strictement isolé) : scalable en pratique jusqu’à des **centaines** de schémas ; au-delà, surveiller **taille du catalogue système**, migrations, ops — pas une limite « dure » du produit Postgres, plutôt **opérationnelle**.
- **Migrations** : outil type **Atlas** (ou Flyway bien cadré) **dans l’artefact** du module, appliquées par un **job / service migrator** avec rôle DB **privilégié**, **pas** par le handler Wasm/Lambda au hasard.
- **Rôle Postgres par module** : droits minimaux (schéma uniquement).

### 3.4 Communication entre modules

- **Bus d’événements** (**Axon + RabbitMQ** déjà en place) : les modules **ne s’appellent pas** en HTTP entre eux ; ils **publient / consomment** des événements déclarés dans le manifeste.
- **L’API / l’hôte** reste le point d’**orchestration synchrone** si besoin (ou query gateway).

### 3.5 Pinning & versions

- **Par tenant** : catalogue « ce tenant utilise `module-foo@1.4.2` » — plusieurs versions du **même** module peuvent coexister pour des tenants différents si le runtime le permet.
- **SDK hôte** (contrat hôte ↔ module) : politique type **N / N-1 / N-2** ; le module déclare **`requiresHostSdk`** (semver **X.Y.Z**, aligné sur **`@portaki/module-sdk`**) — **implémenté** dans les dépôts (voir §6).
- **Downgrade** après migration **DB** : en général **impossible** sans migration de retour explicite ; à anticiper dans le produit (pas un détail Wasm).

### 3.6 « On ne recompile plus le host à chaque module ? »

- **Oui** si le module est un **artefact** (`.wasm` + bundle UI) **chargé à l’exécution** depuis un registre, avec **manifeste** côté tenant.
- **Côté web** : **Module Federation** ou **import ESM dynamique** depuis CDN pour ne pas rebuilder le shell à chaque module.
- **Exception** : changement de **contrat plateforme** (SDK hôte majeur, nouvelles capabilities) → release hôte nécessaire, avec fenêtre de **dépréciation**.

### 3.7 npm / Maven après bascule Wasm ?

- **Modules exécutés** : plus comme **deps Maven** embarquées dans `portaki-api` pour le runtime invité.
- **Développement** : **`@portaki/module-sdk`** (npm) et artefacts **Java SDK** restent pour **DX** (types, build vers Wasm).
- **Core** Portaki : Maven/npm internes **inchangés** pour le code produit hors modules invités.

---

## 4. Microservices : combien, lesquels ?

- **Pas** « 1000 microservices HTTP reliés entre eux ».
- **Oui** à quelques services **bornés** à grande échelle, par exemple :
  - **`portaki-api`** (orchestrateur, auth, API publique) ;
  - **`portaki-module-runtime`** (Wasm, scalable horizontal) ;
  - **`portaki-module-migrator`** (job, rôle DB DDL — **séparé** de l’API pour ne pas donner DDL à l’API) ;
  - **Registre OCI** managé (GHCR, etc.), pas un service custom.

---

## 5. Coûts (ordre de grandeur, discussion mai 2026)

- **Infra** d’un tier Wasm + DB modules + migrator : souvent **centaines d’euros / mois** selon cloud — **faible** vs coût **ingénierie** d’une **plateforme** (ordre de grandeur évoqué : **~75–100k€** one-shot sur plusieurs mois pour une équipe qui va au bout, **hors** POC).
- **ROI** : intérêt quand le **coût de coordination** Maven + releases API devient dominant (souvent autour de **50–100 modules** avec releases fréquentes — ordre de grandeur indicatif).
- **Early stage** (peu de tenants, ~14 modules) : **ne pas** déployer toute l’usine ; **poser les contrats** (manifeste, bus, schémas) coûte peu et évite la dette.

---

## 6. Ce qui est déjà implémenté (mai 2026) — rappel court

Voir détail dans [`MODULE_PLATFORM_PREPARATION.md`](./MODULE_PLATFORM_PREPARATION.md) :

- Champ manifeste **`requiresHostSdk`** (+ alias `requires_host_sdk`), schéma **`portaki-sdk/schema/module.v1.json`**, tous les **`portaki.module.json`** des modules officiels avec **`0.5.0`** aligné sur `portaki-sdk/sdk/module-sdk/package.json`.
- API : **`ModuleManifest`** / réponse registre ; **`/api/v1/healthz`**, **`/api/v1/readyz`**.
- CI : validation optionnelle dans **`portaki-modules/scripts/validate-manifests.mjs`**.

---

## 7. Ce qu’il ne faut **pas** faire trop tôt

- **Kubernetes** « pour être prêt » sans charge : coût apprentissage + ops ; rester sur **12-factor** + Docker + hébergeur simple.
- **Self-host prod** sur machine perso : risques dispo, IP, légal, backups.
- **DDL depuis le runtime module** en prod.

**Signaux raisonnables avant K8s** : plusieurs services à orchestrer, HA multi-AZ, équipe ops, SLA clients.

**Hébergement early** : Railway, Hetzner, Oracle Free Tier, etc. — **produit d’abord**.

---

## 8. Signaux « il est temps de lancer le chantier Wasm / runtime »

- **>50–100 modules** ou releases modules **très** fréquentes qui **forcent** des releases API.
- **Besoin d’isolation** forte (tiers, sandbox, quotas CPU/RAM).
- **Besoin de pinning** fin par tenant + versions multiples en parallèle.
- **Équipe** prête à assumer **ops** d’une plateforme.

---

## 9. POC suggéré (quand tu lances)

1. **Semaine 1** : Wasm embarqué ou petit service + hello world.
2. **Semaine 2** : migrator + un schéma module + Atlas ou Flyway strict.
3. **Semaine 3** : un module réel (ex. petit module existant) en pilote.
4. **Semaine 4** : chargement dynamique UI (Federation ou ESM).

---

## 10. Glossaire rapide

| Terme | Sens ici |
|--------|-----------|
| **Hôte** | Process/API Portaki qui charge ou appelle les modules. |
| **SDK hôte / module-sdk** | Contrat UI invité npm **`@portaki/module-sdk`** ; champ manifeste **`requiresHostSdk`**. |
| **Manifeste** | **`portaki.module.json`** — schéma `module.v1`. |
| **LRU** | Politique d’éviction mémoire dans le runtime Wasm. |
| **OCI** | Format registre conteneurs / artefacts (souvent GHCR). |

---

## 11. Mise à jour de ce mémo

Quand une décision **fige** ou **change** la trajectoire :

1. Éditer ce fichier (section concernée + date en tête de section).
2. Recopier une ligne courte dans le journal du workspace (**`AGENTS.md`** à la racine **Repositories/** si utilisé) ou dans le CHANGELOG du dépôt concerné.
3. Si le changement touche uniquement le pipeline modules, compléter aussi [`MODULE_PLATFORM_PREPARATION.md`](./MODULE_PLATFORM_PREPARATION.md).

---

*Ce fichier est versionné dans **portaki-modules** sous `docs/MODULE_PLATFORM_ARCHITECTURE_MEMO.md`.*
