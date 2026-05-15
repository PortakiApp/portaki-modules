# @portaki/module-trmnl

Module communautaire Portaki pour afficher les informations de séjour sur un écran ePaper [TRMNL](https://trmnl.com).

**Auteur** : [Cyril Colinet](https://github.com/cyrilcolinet) (@cyrilcolinet)  
**Paquet npm** : `@portaki/module-trmnl`  
**Artefact Java** : `app.portaki.module:trmnl-backend`  
**License** : MIT  
**Portaki** : v1.0.0+

Sources : [PortakiApp/portaki-modules](https://github.com/PortakiApp/portaki-modules) — dossier `modules/trmnl/`.

---

## Prérequis

- Appareil TRMNL avec plan **Developer Edition** (webhooks Private Plugin)
- Compte [usetrmnl.com](https://usetrmnl.com)
- Portaki v1.0.0+

---

## Installation

1. Activez le module depuis le dashboard Portaki (**Logement → Modules → TRMNL ePaper**) lorsqu’il sera enregistré dans le catalogue.
2. Côté Java, ajoutez la dépendance `app.portaki.module:trmnl-backend` et exposez un bean `TrmnlModule` (voir ci-dessous).

---

## Intégration hôte (Java)

Le SDK Portaki fournit `GatewayModuleContext` et les annotations `@PortakiModule` / `@PortakiEventHandler`. Les méthodes de `TrmnlModule` sont conçues pour être appelées lorsque l’API Portaki relaie les événements déclarés dans `portaki.module.json` (séjours, checklist).

Instanciation minimale :

```java
import app.portaki.module.trmnl.TrmnlModule;

TrmnlModule trmnl = TrmnlModule.createDefault();
// puis, pour un événement donné :
trmnl.onStayCreated(eventMap, gatewayModuleContext);
```

L’hôte doit enrichir `eventMap` avec des champs **snake_case** documentés (aucun accès base de données dans ce module). Exemples pour le **dashboard hôte** :

| Clé | Description |
|-----|-------------|
| `current_stay_guest` | Prénom ou libellé court du voyageur en cours |
| `current_stay_checkout` | Départ (texte déjà formaté) |
| `next_stay_guest` | Prochain voyageur |
| `next_stay_checkin` | Arrivée (texte formaté) |
| `next_stay_countdown_days` | Entier (jours avant arrivée) |
| `checklist_progress` | 0–100 |
| `alerts_count` | Nombre d’alertes compactes |

**Affichage logement** (`guest_display`) :

| Clé | Description |
|-----|-------------|
| `guest_first_name` | Prénom uniquement |
| `checkin_date`, `checkout_date`, `checkin_time`, `checkout_time` | Chaînes formatées côté Portaki |
| `wifi_ssid`, `door_code` | Codes d’accès |
| `stay_active` | booléen |
| `next_local_event` | optionnel (ex. module *events*) |

### Config optionnelle `trmnl_screen`

Objet JSON dans la configuration du module : fusionné en premier dans le payload, utile pour tests ou champs statiques, puis surchargé par `eventMap`.

---

## Configuration TRMNL

1. [usetrmnl.com](https://usetrmnl.com) → Plugins → New Plugin → **Private Plugin** → stratégie **Webhook**.
2. Copiez l’URL webhook dans Portaki (`webhook_url`, type secret).
3. Collez le markup Liquid depuis `templates/` (inclure `shared.liquid` ou recopier les styles).

Layouts : `full.liquid`, `half_horizontal.liquid`, `half_vertical.liquid`, `quadrant.liquid` pour `host-dashboard/` et `guest-display/`.

---

## Limitations

- Rate limit côté module : **11 requêtes / heure / URL webhook** (marge sous la limite gratuite TRMNL ~12/h).
- Taille payload : viser **< 2 ko** (gratuit) / 5 ko (TRMNL+).
- Rafraîchissement écran TRMNL : 15–90 minutes selon réglages appareil.
- Les échecs webhook sont **journalisés uniquement** (aucune exception vers Portaki).

---

## Contribuer

Voir [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## Licence

MIT © [Cyril Colinet](https://github.com/cyrilcolinet)
