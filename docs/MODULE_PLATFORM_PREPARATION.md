# Préparation plateforme modules (Wasm, scale, ops)

**Mémo détaillé (conversation d’architecture, à relire dans 6 mois)** : [`MODULE_PLATFORM_ARCHITECTURE_MEMO.md`](./MODULE_PLATFORM_ARCHITECTURE_MEMO.md).

Document de synthèse (mai 2026) pour reprendre l’architecture sans rediscuter les options. Il complète les règles déjà en place (`portaki.module.json`, registre GitHub, Axon/RabbitMQ).

## Objectif

Permettre **beaucoup de modules** (ordre 100–1000+) sans :

- graphe de **microservices** HTTP entre modules ;
- **rebuild/redeploy** du monolithe API à chaque bump d’un module invité ;
- **DDL au runtime** depuis du code module (risque audit / rollback).

## Décisions cibles (quand la douleur le justifiera)

| Sujet | Direction recommandée |
|--------|------------------------|
| Artefact module | **Wasm** (ou équivalent sandbox) + registre **OCI** / GHCR, pas JAR Maven ni seul npm dans le classpath hôte |
| Runtime | **Service dédié** `portaki-module-runtime` (pods + LRU), pas tout in-process dans l’API si 1000+ combinaisons module×version |
| Schéma DB | **Base « modules » séparée du core** + **schéma Postgres par module** ; migrations **versionnées** appliquées par un **job** (ex. Atlas), pas par la Lambda/fonction |
| Communication | **Événements** (Axon + RabbitMQ déjà là) ; pas d’appels directs module → module |
| Pinning | **Par tenant** dans le catalogue (`module@version`) ; politique **SDK hôte** N / N-1 / N-2 |
| K8s | **Pas avant** besoin réel (multi-service, HA, scale) ; d’ici là **12-factor** + conteneurs |

## Ce qui est déjà codé dans les dépôts (préparation légère)

1. **`requiresHostSdk` dans `portaki.module.json`**  
   Semver **X.Y.Z** = version minimale de **`@portaki/module-sdk`** (npm) avec laquelle le module guest a été validé. Source de vérité version SDK : `portaki-sdk/sdk/module-sdk/package.json`.  
   Alias accepté côté parseur API : `requires_host_sdk` (snake_case).

2. **Schéma JSON** : `portaki-sdk/schema/module.v1.json` (champ documenté + pattern semver).

3. **Domaine / API** : `ModuleManifest` et `ModuleManifestResponse` exposent `requiresHostSdk` (omis en JSON si vide).

4. **Santé / readiness** : `GET /api/v1/health`, **`/api/v1/healthz`** (liveness), **`/api/v1/readyz`** (probe DB si `DataSource` présent ; sinon `db: skipped`). Public, aligné sécurité avec `/health`.

5. **CI manifestes** : `portaki-modules/scripts/validate-manifests.mjs` vérifie le format de `requiresHostSdk` s’il est présent.

## Règles de conception à respecter dès maintenant (sans nouvelle infra)

- **Pas d’état métier durable en mémoire** dans un module backend : tout en DB / cache / événements.
- **Pas d’import d’un module par un autre** : intégration via **contrat manifeste** + bus.
- **Migrations** : fichiers versionnés dans le dépôt module ; pas de SQL ad hoc non reviewé en prod.
- **Config** : variables d’environnement pour l’hôte ; pas de secrets dans les manifestes.

## Hébergement / coûts (rappel)

- Early stage : **Railway**, **Hetzner**, **Oracle Free Tier**, etc. — priorité **produit** sur **optimisation infra**.
- **Pas de Kubernetes** tant qu’il n’y a pas plusieurs services à orchestrer ou une exigence HA claire.

## Prochaines étapes quand les signaux d’échelle apparaissent

1. POC **Extism** / Wasmtime dans un **petit service** + 1 module pilote.
2. Pipeline **build `.wasm`** + push **GHCR**.
3. Job **Atlas** (ou équivalent) pour schémas `modules/<id>`.
4. **Module Federation** (ou ESM dynamique) côté web pour ne pas rebuilder le shell à chaque module.

Pour toute évolution contractuelle (champ manifeste, SDK), mettre à jour ce fichier et une ligne dans `AGENTS.md` à la racine du workspace **Repositories**.
