# Modules issus du livret « Gîte Le Platet » — specs

Relevé du 2 octobre 2026, à partir d'un livret d'accueil papier de 25 pages (gîte 3 épis,
6 personnes, Saint-Jean-d'Arvey, Savoie) confronté aux 21 modules du dépôt.

Vocabulaire et découpage des surfaces : voir [`guest-surfaces.md`](guest-surfaces.md).
Règle d'ouverture reprise de ce document : **feuille du bas** pour ce qu'on consulte,
**plein écran** pour ce qu'on lit ou qu'on remplit.

Provenance de toutes les surfaces ci-dessous : **spécifié** — rien n'est encore rendu.
Les libellés français entre guillemets sont des propositions, pas des textes existants.

**Périmètre de ce document** : 2 nouveaux modules, 2 extensions. Ce qui en est volontairement
sorti est listé en fin de fichier avec sa raison.

---

## 1. Nouveau module — `safety-shutoffs` (« Coupures & sécurité »)

### Quoi

Où se trouvent le tableau électrique, les vannes d'arrêt d'eau, le robinet de gaz,
l'extincteur, le détecteur de fumée. Ce que le voyageur cherche en panique à 23 h, et qu'aucun
module ne porte : `emergency-contacts` donne des numéros, pas des emplacements.

Le livret source dit « le tableau électrique et disjoncteur principal se trouvent dans le
placard à droite de l'entrée » et « les vannes d'arrêt d'eau froide et d'eau chaude sanitaire
sont accessibles par la trappe supérieure dans WC ». C'est exactement la matière du module.

### Surfaces voyageur

**Carte d'accueil** (`home.card`) — discrète, elle ne doit pas inquiéter en temps normal.

1. `Card` avec icône `ShieldCheck` (à arbitrer avec design) et titre « En cas de problème »
2. `Text` caption — « Électricité, eau, gaz : où couper »
3. `Chevron` vers le détail

**Détail** (`explore.detail`, **plein écran**) — on le lit sous stress, donc gros blocs, pas de densité.

1. `InfoBanner` optionnel — la consigne générale de l'hôte, si elle est saisie
2. `List` d'organes, un `ListItem` par entrée, chacun composé de :
   - `Text` title — le nom : « Tableau électrique »
   - `Text` body — l'emplacement : « Placard à droite de l'entrée »
   - `Image` optionnel — la photo de l'organe, c'est ce qui fait gagner les trente secondes
   - `Text` caption optionnel — la consigne : « Disjoncteur principal en haut à gauche »
3. `Divider`
4. `EmergencyButton` — « Appeler l'hôte », action téléphone vers le numéro de l'hôte

> Le primitif `EmergencyButton` existe déjà (`label` + `action`). À confirmer avec design :
> est-ce le bon registre visuel ici, ou est-il réservé au 112 ?

### Config hôte (surface `main`)

| Champ | Nature | Note |
|---|---|---|
| `shutoffs` | `#[field(structured, required)]` — liste de lignes | voir ci-dessous |
| `general_note` | `#[field]` `I18nText` | consigne commune, affichée en bandeau |

Une ligne `ShutoffRow` : `id`, `kind` (select : électricité / eau / gaz / extincteur /
détecteur de fumée / autre), `title` `I18nText`, `location` `I18nText`, `instruction`
`I18nText` optionnel, `photo` optionnel (`ImageUpload`).

**Pas de slots fixes** — adopter d'emblée le clamp dynamique de `rules` / `ical-sync`, pas le
`const N_SLOTS: usize = 6` des autres modules (voir § 5).

### listing.json

`category: "stay"` · `capabilities: ["core.storage", "core.images"]` · icône `ShieldCheck`
· `configItems` : les deux champs ci-dessus.

### Couches touchées

- **Module** (Rust) : tout, c'est un module neuf.
- **Guest renderer** : rien de neuf. `Card`, `List`, `ListItem`, `Text`, `Image`, `Divider`,
  `InfoBanner`, `EmergencyButton` existent tous.
- **Plateforme** : rien, sauf le point images ci-dessous.
- **Dashboard** : rien de spécifique.

### Ouvert

- **`core.images` n'a jamais été consommé par un module.** La capacité est au catalogue, le
  primitif `ImageUpload` et le primitif `Image` existent, mais **aucun des 21 modules ne la
  déclare aujourd'hui**. Ce module en serait le premier usage réel : prévoir que le chemin
  upload → stockage → URL servie au livret n'est pas rodé. À dérisquer avant de s'engager, ou
  à livrer en v1 sans photo.
- Le bouton « Appeler l'hôte » duplique-t-il `emergency-contacts` ? Si oui, le retirer et
  renvoyer vers ce module.

### Coût i18n

≈ 10 clés nouvelles (titres de surfaces, libellés de formulaire, états vides, les 6 valeurs du
select `kind`) × 10 locales. Bundle complet obligatoire avant `portaki check`.

---

## 2. Nouveau module — `trails` (« Randonnées »)

### Quoi

Les itinéraires au départ du logement, classés par difficulté. `local-guide` fait des adresses
ponctuelles sur une carte ; une randonnée est une boucle avec un dénivelé, une durée et un tracé.
Modèle de données différent, et c'est le contenu que vend un gîte de montagne.

Le livret source en liste neuf sur trois niveaux : « Ballade en partant du gîte » (sentier des
Vignes, les Trois Sapins, cascade de la Doria), « Plus long » (trou d'eau de la Doria, grotte à
Carret, vestiges archéologiques du Peney), « Plus technique » (Mont Peney, croix du Nivolet).

### Surfaces voyageur

**Carte d'accueil** (`home.card`)

1. `Card` icône `Mountain`, titre « Randonnées au départ du gîte »
2. `Chips` — un chip par niveau présent, avec le compte : « Facile · 3 », « Sportif · 2 »
3. `Chevron`

**Détail** (`explore.detail`, **plein écran** — on le lit, on le compare)

1. `FilterBar` ou `Chips` sélectionnables — filtre par niveau
2. `List` d'itinéraires, un `ListItem` par randonnée :
   - `Text` title — le nom
   - `Badge` — le niveau, couleur par difficulté (à cadrer par design)
   - `KeyValue` en ligne — durée, distance, dénivelé
   - `Text` caption — la description courte de l'hôte
   - `Chevron` vers la fiche
3. `Link` optionnel — le lien « tous les sentiers de la commune » de l'hôte

**Fiche d'un itinéraire** (`explore.item`, **plein écran**)

1. `Image` optionnel — photo d'ouverture
2. `KeyValue` — durée, distance, dénivelé, type (boucle / aller-retour)
3. `Text` — la description
4. `Map` — **le point de départ en marqueur**, voir la réserve ci-dessous
5. `Button` — « Ouvrir dans l'app de rando » (lien externe : trace GPX, Visorando, IGN…)

> **Réserve de périmètre.** Le primitif `Map` porte `markers`, `viewport`, `clustering`,
> `radius` — **il n'a aucun champ de tracé**. Afficher une polyline suppose soit un champ
> `polylines` sur `Map`, soit un primitif nouveau. **C'est la seule vraie pièce de guest
> renderer de ce document, et design doit savoir qu'il dessine un composant neuf.**
>
> **Proposition v1 : pas de tracé.** Marqueur de départ + bouton vers une trace externe.
> Ça couvre l'usage réel — on ne suit pas un GPX dans un livret, on l'ouvre dans son app de
> rando — et ça supprime la dépendance au renderer. Le tracé devient une v2 assumée.

### Config hôte (surface `main`)

| Champ | Nature |
|---|---|
| `trails` | `#[field(structured, required)]` — liste |
| `community_link` | `#[field(kind = "url")]` |

Une ligne `TrailRow` : `id`, `title` `I18nText`, `level` (select : facile / moyen / sportif),
`duration_min` (nombre), `distance_km` (nombre), `ascent_m` (nombre), `shape` (select : boucle /
aller-retour), `description` `I18nText`, `start_lat` / `start_lng` (via `AddressMapPicker`,
comme `local-guide`), `external_url` (`kind = "url"`), `photo` optionnel.

Clamp dynamique, pas de slots fixes.

### listing.json

`category: "around"` · `capabilities: ["core.storage", "core.images"]` · icône `Mountain`.

### Couches touchées

- **Module** : tout.
- **Guest renderer** : rien **si on retient la v1 sans tracé**. Sinon, un champ `polylines` sur
  `Map` côté SDUI + son rendu dans le livret.
- **Plateforme / dashboard** : rien.

### Ouvert

- Les trois niveaux sont-ils figés, ou l'hôte nomme-t-il les siens ? Le livret source en a trois
  mais les appelle « Ballade », « Plus long », « Plus technique » — ce ne sont pas des niveaux
  de difficulté standards. Figer trois valeurs traduites me semble plus sûr qu'un texte libre.
- Faut-il croiser avec `weather` (« sentier déconseillé par temps de pluie ») ? Hors v1.

### Coût i18n

≈ 14 clés × 10 locales.

---

## 3. Extension — `appliances` : étapes illustrées

### Le manque

Le modèle actuel est `devices[].steps: [texte]` plus un `manualUrl`. Or quatre pages du livret
source sont **des schémas purs** : les trois positions d'une fenêtre oscillo-battante, l'ouverture
d'un parasol en cinq vues, les réglages d'une chaise Stokke, le montage du Baby Set. Du texte ne
remplace pas un schéma. Et la phrase « toutes les notices sont regroupées dans la boîte rouge sur
l'étagère du salon » dit exactement le second manque : l'hôte n'a nulle part où déposer ses PDF.

### Le delta

- **Une étape devient `{ text, image? }`** au lieu d'une chaîne. Côté voyageur, chaque étape est
  un `Stack` : `Text` body + `Image` optionnel. Rien de neuf dans le renderer.
- **Le module déclare `core.images`** en plus de `core.storage` (aujourd'hui il ne déclare que
  `core.storage`). Même réserve qu'au § 1 : premier usage réel de la capacité.
- **`manualUrl` devient un fichier** plutôt qu'une URL, ou accepte les deux. À arbitrer :
  héberger des PDF est un autre sujet que des images.
- **Migration** : l'ancien `steps: ["…"]` se relit en `[{ text: "…" }]` via le `legacy()` du
  module, comme `facility-hours` le fait déjà pour `facilities_json`.

### Couches touchées

Module seul, plus la réserve `core.images`. Pas de guest renderer, pas de plateforme.

---

## 4. Extension — `waste-recycling` : points d'apport et composteur

### Le manque

Le modèle actuel est `bins[]` (titre, contenu, couleur) + sept booléens de jours de collecte +
une note. Il suppose un ramassage en porte-à-porte. Le livret source n'en a pas : il y a **deux
points d'apport géolocalisés** (parking du cimetière, parking de la salle des fêtes, l'un avec le
verre en plus) et **un composteur** avec sa liste à mettre / ne pas mettre. En rural et en
montagne c'est la norme, pas l'exception.

### Le delta

- **`dropoff_points[]`** : `title`, `lat`/`lng` (`AddressMapPicker`), `accepts` (multi-select :
  ordures ménagères / emballages / verre / papier), `note` optionnelle.
  Côté voyageur, un bloc `Map` avec un marqueur par point — `markers` existe, rien de neuf.
- **`compost`** : un toggle d'activation, `location` `I18nText`, et deux `I18nText`
  « à mettre » / « à ne pas mettre ». Côté voyageur, deux `BulletList` sous un `Section`,
  ou un `Accordion` si la place manque.
- Les jours de collecte deviennent **facultatifs** : aujourd'hui la surface les suppose.
  Un logement sans ramassage ne doit pas afficher une ligne vide.

### Couches touchées

Module seul. Aucun primitif nouveau.

---

## 5. Hors design — la limite à 6, côté hôte

Cinq modules figent leur nombre de lignes en dur : `local-guide` (`SPOT_SLOTS`),
`waste-recycling` (`BIN_SLOTS`), `emergency-contacts` (`CONTACT_SLOTS`), `events`
(`EVENT_SLOTS`), `facility-hours` (`FACILITY_SLOTS`) — tous à **6**. `consumables` et
`access-guide` sont à 8.

Ce livret en déborde partout : 8 commerces rien que pour le village plus l'annuaire communal,
8 numéros utiles, 4 lacs, 3 stations de ski.

`rules` (`ITEM_SLOTS = 12` avec `clamp(1, ITEM_SLOTS)`) et `ical-sync`
(`CALENDAR_SLOTS = 20` avec `min`) ont déjà le bon motif : un nombre de lignes dynamique borné.

**Décision à prendre** : généraliser ce motif module par module, ou le remonter dans l'aide
`structured` du SDK pour que tous en héritent. La seconde option est une correction unique qui
débloque les cinq modules ; c'est ton arbitrage. Aucun impact design, aucun impact voyageur.

---

## 6. Question ouverte — les couchages

`Property` porte `name`, `address`, `lat`/`lng`, `checkinTime`, `checkoutTime`, `timezone`,
`themeId`. **Ni capacité, ni chambres, ni lits.** Or « 3 chambres, 6 personnes, un 160 séparable
en deux 80, des lits superposés » est la première question d'un voyageur avant d'arriver, et la
donnée servirait aussi au `pre-arrival-form` et à la taxe de séjour.

Je n'ai pas de recommandation ferme, parce que la question n'est pas « quel module » mais
**où ça s'affiche** : une fiche logement portée par la coque du livret, un module dédié, ou
simplement `sections` ? Le modèle de données appartient à la plateforme dans les trois cas.
À trancher avant d'écrire quoi que ce soit.

---

## 7. Pourquoi ces modules d'abord — l'import de livret

Ce document est né d'un livret papier, et l'idée qui en découle est d'en faire la rampe d'accès du
produit : l'hôte dépose le livret qu'il a déjà, Portaki le lit et **propose** une configuration
qu'il valide champ par champ.

**Ce n'est pas le chantier de cette itération**, et le sujet a désormais son propre handoff :
`docs/handoff-import-livret.md` du dépôt `portaki-platform` — les trois couches d'import, le
relevé de l'existant (ni OCR ni lecture de PDF nulle part dans la plateforme), le flux, les écrans
et les cinq règles pour design.

Ce qu'il faut en retenir **ici**, parce que ça justifie la priorité de ce document-ci : la valeur
de l'import est proportionnelle au nombre de modules qu'il sait remplir. Ce livret montre que le
modèle actuel ne sait tenir ni les organes de coupure, ni les randonnées, ni le composteur, ni les
points d'apport volontaire. Un import construit avant ces modules importe dans un modèle qui ne
peut pas accueillir ce qu'il lit, et se reprend à chaque module ajouté. Construit après, il naît
avec la couverture. **Les modules d'abord.**

---

## Volontairement hors de ce document

- **Extras payants.** Aucune capacité de paiement n'existe, dans le SDK comme dans la
  plateforme. C'est la tranche 3 d'ADR-0022 (`portaki-platform`, `docs/adr/0022-portaki-ne-detient-jamais-les-fonds-d-un-voyageur.md`)
  et rien ne peut être construit ni dessiné avant que l'ADR soit accepté. Spécifier une surface
  voyageur maintenant ferait dessiner un écran mort.
- **Inventaire et état des lieux.** Vrai manque — `checklist` fait des cases binaires avec photo,
  pas de quantités, pas de colonne entrée/sortie, pas de signature — mais c'est un document
  contractuel lié à la caution, donc un chantier à part. Second rang.
- **Histoire du bâtiment, éco-construction, zones non accessibles, consignes de température.**
  `sections` suffit.
- **Classement et numéro d'enregistrement.** Déjà modélisés côté plateforme
  (`classement_meuble`, profil réglementaire) ; reste un sujet d'affichage, pas de module.
- **Horaires d'arrivée et de départ.** Déjà sur `Property`.
- **Ligne de bus du village.** `train` est en v0.1 : constantes Rust en dur, aucun éditeur hôte.
  Rien à y greffer avant son passage en config / Navitia.
