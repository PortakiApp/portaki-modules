# Schéma des README modules (landing + dépôt)

Chaque module sous forme de package npm (`train/`, `events/`, …) utilise **les mêmes sections** dans son `README.md`. Cela permet :

- une lecture homogène dans le monorepo ;
- un copier-coller rapide vers la **landing** (hero, cartes « fonctionnalités », FAQ).

Remplace les portions entre `⟨ ⟩`.

---

```markdown
# ⟨Nom lisible du module⟩

> **⟨Une phrase d’accroche pour la landing⟩** — ⟨sous-titre optionnel⟩.

## Public cible

⟨qui bénéficie du module : voyageurs, propriétaires, cas d’usage⟩.

## Ce que ça apporte

- ⟨bénéfice 1⟩
- ⟨bénéfice 2⟩
- ⟨bénéfice 3⟩

## Fiche technique

| Champ | Valeur |
|--------|--------|
| **Package npm** | `⟨@portakiapp/module-…⟩` |
| **Identifiant `id`** | `⟨string unique⟩` |
| **Slot navigation** | `section` (standard Portaki) |
| **Icône** | `⟨nom icône⟩` |
| **Visibilité** | ⟨statuts séjour / `visibleOnStatus` / toujours visible⟩ |
| **Carte / carte overlay** | ⟨oui / non / `mapOverlay`⟩ |

## Intégration Portaki

⟨comment l’app guest charge le module : import du default export, enregistrement dans le shell, configuration propriété / séjour⟩.

## Données & API

⟨endpoints guest, événements Axon, champs `property` / `stay` utilisés⟩.

## Développement local

Depuis la racine du monorepo :

```bash
pnpm install
```

Ce package dépend de **`@portakiapp/module-sdk`** (workspace). Voir le [README du SDK](../packages/module-sdk/README.md).

## Licence

AGPL-3.0 — voir le fichier `LICENSE` à la racine du dépôt si présent, sinon le champ `license` du `package.json`.
```

---

### Variante « SDK » (`packages/module-sdk`)

Même principe, mais les sections **Ce que ça apporte** et **Fiche technique** décrivent le **contrat** (`definePortakiModule`, types) plutôt qu’une fonctionnalité métier. Référencer ce fichier pour garder la même hiérarchie de titres.
