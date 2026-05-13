# @portaki/module-ical-sync

Module **hôte** : synchronisation de flux iCal (lien d’export Airbnb, Booking, etc.).  
La logique de fetch, parsing et fournisseurs vit dans le **backend Java** de ce module (`ical-sync-backend`) ; ce paquet npm expose le manifeste et un panneau `renderHost` optionnel.

## Portaki

- Activer le module au niveau **Organisation → Modules**.
- Par logement : **Modules** → champs **Lien du calendrier principal** (et optionnellement second lien) → **Synchroniser** (`POST /api/v1/properties/{id}/modules/ical-sync/sync`).

### Comportement Airbnb

- Les créneaux bloqués / indisponibilités exportés avec le titre **Reserved** (ou **Réservé**) ne sont **pas** traités comme des réservations importées.
- L’ancienne configuration en JSON (`feeds_json`) est encore lue si les nouveaux champs URL sont vides (migration).

## Développement

```bash
pnpm install
```

Voir `portaki.module.json` pour la fiche catalogue.
