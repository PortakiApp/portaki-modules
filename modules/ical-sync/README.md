# @portaki/module-ical-sync

Module **hôte** : synchronisation de flux iCal (lien d’export Airbnb, Booking, etc.).  
La logique de fetch, parsing et fournisseurs vit dans le **backend Java** de ce module (`ical-sync-backend`) ; ce paquet npm expose le manifeste et un panneau `renderHost` optionnel.

## Portaki

- Activer le module au niveau **Organisation → Modules**.
- Par logement : **Modules** → flux JSON → **Synchroniser** (API hôte générique `POST /api/v1/properties/{id}/modules/ical-sync/sync` — `ical-sync` est l’identifiant du module).

## Développement

```bash
pnpm install
```

Voir `portaki.module.json` pour la fiche catalogue.
