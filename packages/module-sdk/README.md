# SDK modules Portaki (`@portaki/module-sdk`)

> **Contrat TypeScript pour tous les modules UI guest** — une seule fonction `definePortakiModule` et des types partagés pour garder le shell guest typé.

## Public cible

Développeurs qui créent ou maintiennent des packages `@portaki/module-*` consommés par l’application guest React.

## Ce que ça apporte

- Alignement des props (`property`, `stay`, `lang`) entre modules.
- Déclaratif : métadonnées (`id`, `label`, `icon`, `navSlot`) au même endroit que le rendu.
- Point d’entrée unique pour documenter les extensions (`mapOverlay`, `visibleOnStatus`, etc.).

## Fiche technique

| Champ | Valeur |
|--------|--------|
| **Package npm** | `@portaki/module-sdk` |
| **Rôle** | Types + helper `definePortakiModule` |
| **Pair dependency** | `react >= 18` |
| **Point d’entrée** | `./src/index.ts` |

## API essentielle

- **`definePortakiModule(definition)`** — retourne le même objet ; sert de marqueur et contrat pour le bundle guest.
- **`ModuleRenderContext`** — `lang`, `property`, `stay` optionnel.
- **`PortakiModuleDefinition`** — champs `id`, `label`, `icon`, `navSlot`, `render`, options `visibleOnStatus`, `mapOverlay`, `mapMarkers`.

## Intégration Portaki

Chaque module métier fait :

```ts
import { definePortakiModule } from '@portaki/module-sdk'

export default definePortakiModule({
  id: '…',
  label: { fr: '…', en: '…' },
  icon: '…',
  navSlot: 'section',
  render: ({ property, stay, lang }) => { … },
})
```

L’app guest importe le **default export** et enregistre le module dans son registre interne.

## Données & API

Pas d’appel réseau dans ce package : uniquement des types et une fonction pure.

## Développement local

À la racine du monorepo :

```bash
pnpm install
```

## Licence

AGPL-3.0 — voir le `package.json` du package.
