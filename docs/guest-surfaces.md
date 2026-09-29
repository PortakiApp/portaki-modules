# Les surfaces voyageur, module par module

Ce document décrit **ce que le voyageur voit** dans le livret, écran par écran, pour chacun
des modules. Il est écrit pour être dessiné : chaque bloc est nommé dans son ordre de
lecture, avec le vrai texte français quand il existe.

**19 modules** ont au moins une surface voyageur, pour **45 surfaces** au total.

---

## Comment lire ce document

### La provenance de chaque surface

Chaque surface porte une mention de provenance. Elle dit à quel point le contenu décrit ici
est certain.

- **Aperçu** — la surface a été lue dans un aperçu rendu, c'est-à-dire l'écran réel avec son
  contenu français déjà résolu. Ce qui est écrit est ce que le voyageur voit. **28 surfaces.**
- **Code** — la surface n'a pas d'aperçu ; elle a été reconstituée en lisant le programme qui
  la fabrique. L'ordre des blocs et les libellés fixes sont sûrs ; les contenus saisis par
  l'hôte sont décrits par leur nature, pas par un exemple. **17 surfaces.**

Quand un libellé n'existe que sous forme de clé de traduction et que le texte français n'a pas
pu être vérifié, il est signalé comme tel. Sinon, **tous les textes entre guillemets sont les
libellés français réels** du dépôt.

### Les quatre familles de surfaces

Le vocabulaire revient d'un module à l'autre :

| Surface | Où elle apparaît | Forme |
|---|---|---|
| **Carte d'accueil** (`home.card`) | Sur la page d'accueil du livret, pendant le séjour | Une carte, avec une icône, un titre et quelques lignes ; souvent cliquable vers le détail |
| **Détail** (`explore.detail`, `explore.sheet`, `explore.forecast`, `explore.item`) | Ouvert depuis la carte | Soit une **feuille** qui remonte du bas (bottom sheet), soit une **page plein écran**. Le titre et l'en-tête sont posés par le livret, pas par le module : le module ne dessine que le corps |
| **Carte avant l'arrivée** (`upcoming.card`) | Sur la frise d'avant-séjour | Une carte volontairement minuscule : icône, titre, **une seule ligne** de valeur |
| **Carte de fin de séjour** (`post-stay.card`) | Sur l'écran d'après-séjour | Même forme qu'une carte d'accueil |
| **Formulaire** (`guest.form`) | Ouvert en surimpression depuis la carte | Un formulaire nu — aucun cadre de carte autour |

Deux présentations d'ouverture existent, et le choix n'est pas cosmétique :
**feuille du bas** pour les contenus courts qu'on consulte, **plein écran** pour ceux qu'on
lit ou qu'on remplit. Le tableau de chaque module précise laquelle.

---

## Les états communs

Ce sont les formes qui reviennent partout. Les dessiner une fois sert dix-neuf modules.

### 1. L'état vide « rien à montrer » — très homogène

Onze modules utilisent exactement la même forme : un **bloc d'état vide** avec une icône, un
titre court et une phrase de description. Elle **remplace entièrement** la surface — carte
comme détail, le même bloc dans les deux.

Un point important pour la rédaction : ces textes sont **écrits pour le voyageur, pas pour
l'hôte**. Ils ne disent jamais « configurez ce module » ; ils disent ce qui va arriver.

| Module | Titre | Description |
|---|---|---|
| Accès & parking | « Consignes d'arrivée à venir » | « Votre hôte n'a pas encore indiqué comment entrer dans le logement. Les consignes s'afficheront ici. » |
| Urgences & utiles | « Aucun numéro pour l'instant » | « En cas d'urgence vitale, composez le 112. » |
| Parking VE | « Recharge : informations à venir » | « Votre place et vos codes s'afficheront ici. » |
| Événements | « Aucun événement à venir » | « Rien n'est prévu autour du logement pour le moment. » |
| Horaires & accès | « Pas encore d'horaires » | « Les horaires des équipements s'afficheront ici. » |
| Bons plans | « Pas encore de bons plans » | « Votre hôte n'a pas encore partagé ses adresses. » |
| Tri & déchets | « Consignes de tri à venir » | « Les bacs et les jours de collecte s'afficheront ici. » |
| Wi-Fi & codes | « Wi-Fi bientôt disponible » | « Les informations de connexion ne sont pas encore renseignées. » |
| Votre avis | « Pas d'avis à laisser ici » | « Votre hôte ne recueille pas d'avis depuis ce livret. » |
| Règlement intérieur | « Aucun règlement pour le moment » | « L'hôte n'a pas encore publié de règles pour ce logement. » |
| Bienvenue (sections) | « Rien pour le moment » | « Votre hôte n'a encore rien ajouté ici. » |

À dessiner une fois, à décliner par l'icône du module.

### 2. Les états vides qui divergent — et c'est voulu, sauf une fois

Cinq modules ne prennent pas cette forme. Quatre ont une bonne raison, un seul sort du rang
sans en avoir.

- **Appareils** garde la carte et son en-tête, et pose le bloc d'état vide **à l'intérieur**.
  C'est le seul module qui fasse ça. Deux variantes : dans la carte d'accueil, « Rien en avant
  pour l'instant » / « Ouvrez la liste pour voir tous les appareils disponibles » ; dans la
  page complète, « Aucun appareil » / « Aucun guide appareil n'est disponible pour ce séjour. »
  **À dessiner : un état vide imbriqué dans une carte.**
- **Checklists**, **Consommables** et **Votre avis** gardent aussi la carte, mais y mettent
  **une simple ligne de texte** au lieu d'un bloc d'état vide — « Rien à cocher pour ce
  séjour. », « Pour signaler un manque, contactez directement votre hôte. », « Vous pourrez
  laisser votre avis dès votre arrivée. » L'idée est cohérente (la carte reste, elle existe
  déjà dans le parcours), la forme du message ne l'est pas : c'est du texte nu, sans icône ni
  titre.
- **Les trois modules de formulaire** (Signaler un problème, Objets perdus ou trouvés,
  Pré-arrivée) n'ont **pas** d'état vide : leur carte propose toujours d'ouvrir le formulaire,
  qu'il y ait ou non des déclarations à lister. C'est le bon choix — la surface est une action,
  pas un contenu.
- **Météo** prend bien le bloc d'état vide commun, avec l'icône nuage-soleil et le titre
  « **Météo indisponible pour ce logement** », mais **deux descriptions distinctes** selon la
  cause : « Les prévisions ne sont pas proposées pour ce logement. » quand le service est coupé
  ou que les données n'arrivent pas, « La météo s'affichera dès que l'adresse du logement sera
  précisée. » quand l'adresse manque. Un même bloc, deux secondes lignes.

**Recommandation pour le designer :** aligner les trois cartes à une ligne (Checklists,
Consommables, Votre avis) sur la forme du bloc d'état vide imbriqué d'Appareils. Il n'y a
aucune raison lisible pour deux formes.

### 3. L'état d'erreur « pas pu charger » — homogène, mais pas dessiné par les modules

Douze modules définissent un couple titre + description d'erreur, et **beaucoup ajoutent une
troisième ligne** : « Réessayez dans un instant ». Le patron est toujours le même : un nom de
module suivi de « temporairement indisponible ».

| Module | Titre | Description | 3e ligne |
|---|---|---|---|
| Accès & parking | « Accès temporairement indisponible » | « Les consignes n'ont pas pu être chargées » | « Réessayez dans un instant » |
| Urgences & utiles | « Contacts temporairement indisponibles » | « Les numéros n'ont pas pu être chargés » | oui |
| Parking VE | « Parking VE temporairement indisponible » | « Les informations parking n'ont pas pu être chargées » | oui |
| Événements | « Événements temporairement indisponibles » | « La liste n'a pas pu être chargée » | oui |
| Horaires & accès | « Horaires temporairement indisponibles » | « Les horaires n'ont pas pu être chargés » | oui |
| Bons plans | « Bons plans temporairement indisponibles » | « Les lieux n'ont pas pu être chargés » | oui |
| Tri & déchets | « Tri temporairement indisponible » | « Les consignes n'ont pas pu être chargées » | oui |
| Wi-Fi & codes | « Wi-Fi temporairement indisponible » | « Les informations réseau n'ont pas pu être chargées » | oui |
| Votre avis | « Avis temporairement indisponibles » | « La page d'avis n'a pas pu être chargée » | oui |
| Signaler un problème | « Signalement temporairement indisponible » | « Le formulaire n'a pas pu être chargé. » | **non** |
| Objets perdus ou trouvés | « Signalement temporairement indisponible » | « Le formulaire n'a pas pu être chargé. » | **non** |
| Pré-arrivée | « Formulaire temporairement indisponible » | « Le formulaire n'a pas pu être chargé. » | **non** |
| Météo | « Météo temporairement indisponible » | « Les conditions n'ont pas pu être chargées. Réessayez dans un instant. » | **fondue dans la description** |

Deux écarts à noter, tous deux mineurs mais visibles :
les trois modules de formulaire **terminent leur description par un point** et n'ont pas de
troisième ligne ; Météo **replie** la troisième ligne dans la description.

**Point important pour le dessin :** ces messages d'erreur sont **fournis par les modules mais
affichés par le livret lui-même**, pas par le module. Il faut donc **un seul gabarit d'erreur**,
dans lequel les trois lignes viennent du module. Idem pour les états « module désactivé » et
« module incomplet », que les modules ne dessinent pas du tout.

### 4. L'aperçu et le détail : la même liste, deux densités

C'est le mécanisme le plus structurant du livret, et il concerne **huit modules** (Accès &
parking, Urgences & utiles, Événements, Horaires & accès, Bons plans, Tri & déchets, Wi-Fi &
codes, Météo).

La carte d'accueil et l'écran de détail affichent **la même liste, construite par le même
code**, avec un interrupteur « enrichi ou non ». Concrètement, le détail **ajoute** :

- le **bandeau d'information** en tête (sécurité, avertissement, note générale) — presque
  toujours absent de la carte ;
- la **carte géographique**, quand il y en a une ;
- les **lignes secondaires** sous chaque élément (notes, détails, précisions) ;
- les **liens** sortants dans chaque ligne.

Et surtout, une différence de **forme des lignes** : dans la carte d'accueil de
**Horaires & accès**, chaque équipement est une simple **paire libellé / valeur** ; dans le
détail, il devient une **ligne de liste** avec sous-titre et lignes de texte en dessous. Ce
n'est pas la même primitive : à dessiner comme deux composants distincts.

Autre variante de forme, dans **Événements** et **Bons plans** : quand une ligne porte un lien
sortant, la carte d'accueil rend **la ligne entière cliquable** (elle part vers le lien),
tandis que le détail garde la ligne inerte et ajoute **un lien explicite** en dessous
(« Ouvrir le lien »). Même contenu, deux affordances opposées.

### 5. Les secrets à révélation programmée

Trois modules cachent des valeurs jusqu'à une date choisie par l'hôte : **Accès & parking**
(code de boîte à clés, digicode, code parking, code de secours de serrure connectée),
**Parking VE** (code barrière, PIN de la borne), **Wi-Fi & codes** (mot de passe).

Deux états à dessiner, toujours au même endroit dans la page :

- **Caché.** La valeur est remplacée par un **masque de six points** (`••••••`), en caractères
  à chasse fixe comme la vraie valeur. **Le bouton « Copier » disparaît** — il n'existe que
  quand la valeur est visible. En tête de surface, un **bandeau d'information** apparaît :
  titre « Codes pas encore disponibles » (« Mot de passe pas encore disponible » pour le
  Wi-Fi), message calculé — soit « Les codes seront disponibles à l'approche du check-in. »,
  soit « Disponible à partir du » suivi d'une date au format `JJ/MM/AAAA à HH:MM`.
- **Révélé.** La valeur en clair, et le bouton « Copier » sous elle.

Le bandeau n'apparaît **que s'il y a effectivement un secret à cacher**. Un logement sans
aucun code ne le montre jamais.

L'hôte choisit parmi quatre moments : « Toujours visible », « Veille à 16 h » (recommandé),
« 24 h avant l'arrivée », « À l'heure du check-in ».

### 6. Les valeurs à copier

Partout où une valeur se copie (mot de passe Wi-Fi, code barrière, PIN de borne), la forme est
identique : une **paire libellé / valeur en chasse fixe**, puis **un bouton de contour** juste
en dessous, libellé « Copier <la chose> ». Le retour se fait par un message éphémère (toast) :
« Mot de passe copié », « Code barrière copié », « PIN borne copié ».

### 7. Les cartes géographiques

Quatre surfaces en montrent une (Accès & parking, Événements, Bons plans — détail uniquement).
Elles sont toujours **statiques et non manipulables** : ni zoom, ni déplacement. La raison est
explicite dans le code — ces surfaces sont des feuilles qui défilent, et une carte manipulable
volerait le geste de défilement.

Deux types de repères : le **logement** et les **points d'intérêt**. Quand les deux coexistent
(Bons plans), le logement entre dans le calcul du centre — sans lui, le voyageur lit des points
sans savoir d'où il part.

Une carte **n'apparaît pas du tout** si aucun élément n'a de coordonnées. Une carte vide vaut
moins que pas de carte.

### 8. Les mentions obligatoires

**Bons plans** est le seul module à porter des liens rémunérés. Deux mentions, toujours sous
la liste, dans toutes les langues :

- « Liens partenaires : Portaki peut percevoir une commission, sans surcoût pour vous. »
  (activités)
- « Billets, prix et notes fournis par Tiqets. » puis « Liens partenaires : une commission peut
  être perçue sur les réservations, sans surcoût pour vous. » (billetterie)

Elles se lisent **avant le clic**, pas après. À dessiner comme des légendes discrètes mais non
tronquées.

---

## Accès & parking

> *À quoi sert la surface :* le moyen d'entrée (boîte à clés, digicode, serrure connectée,
> remise des clés en main propre…), l'accès à l'immeuble, le parking et le parcours jusqu'à la
> porte, étape par étape, avec une carte et une vidéo facultative. Les codes apparaissent selon
> le calendrier choisi par l'hôte ; avec une serrure connectée liée, un bouton ouvre la porte.

**3 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **voiture**, titre « Accès & parking ». La carte entière ouvre le détail **en
plein écran**.

Contenu, dans l'ordre :

1. Le **bandeau des codes non révélés**, si des codes existent et ne sont pas encore visibles
   (voir *États communs §5*).
2. La **carte géographique**, centrée sur le lieu de rendez-vous s'il est renseigné, sinon sur
   le logement. Repère « Logement ». Absente si aucune coordonnée n'est connue.
3. « **Adresse** » + l'adresse complète saisie par l'hôte.
4. « **Moyen d'accès** » + le moyen choisi, parmi sept valeurs : « Boîte à clés »,
   « Code portail » / « Code immeuble » / « Code appartement » (selon ce que le code ouvre),
   « Serrure connectée », « Remise en main propre », « Réception » / « Gardien / loge »,
   « Accueil par l'hôte », « Autre ».
5. Les champs propres au moyen choisi. Selon le cas : « Emplacement » (de la boîte),
   « Code boîte à clés », « Code », « Code de secours », « Lieu de rendez-vous », « GPS »,
   « Horaire », « Contact », « Emplacement » (du comptoir), « Horaires »,
   « Indication d'arrivée ». **Toute ligne vide est retirée**, jamais affichée vide.
6. Pour une **serrure connectée reliée à un module** et **seulement si les codes sont
   révélés** : deux boutons, « **Déverrouiller** » (plein) et « **Obtenir mon accès** »
   (contour).
7. « **Digicode** » et « **Interphone** », si l'hôte a activé la couche immeuble.
8. « **Parking** » (texte libre) et « **Code parking** », si la couche parking est activée.
9. Un bouton de contour « **Ouvrir dans Maps** », vers le plan du parking s'il existe, sinon
   vers les coordonnées du rendez-vous, sinon vers celles du logement.

**Ce qui varie :** le moyen d'accès change entièrement le bloc 5 — ce sont sept mises en page
à prévoir. Les couches immeuble et parking sont chacune optionnelles. La carte, le bouton Maps
et les boutons de serrure peuvent tous être absents.

### Détail — *Aperçu* — plein écran

Le même corps que la carte, **plus** trois choses :

1. **En tête**, un bandeau d'information titré « **À savoir** », avec la note générale de
   l'hôte. Exemple lu : *« Merci de refermer la boîte à clés après usage. »*
2. Les **instructions complémentaires** du moyen d'accès, sous le libellé « **Instructions** ».
   Exemple lu : *« Composez le code, tirez le volet vers le bas : les clés sont sur le
   crochet. »* Ainsi que la « **Note immeuble** ».
3. **En pied**, le parcours d'arrivée :
   - un lien « **Voir la vidéo d'arrivée** », si l'hôte en a mis une (liens sécurisés
     uniquement) ;
   - puis les **étapes**, jusqu'à six. Chaque étape est une ligne de liste avec un **titre**,
     une **pastille** qui dit son type — « Parking », « Porte », « Ascenseur » ou « Autre » —
     et une ligne de détail en petit.

Exemple d'aperçu complet lu, dans l'ordre : bandeau « À savoir », carte (Cannes), « Adresse »
= *12 rue des Oliviers, 06400 Cannes*, « Moyen d'accès » = *Boîte à clés*, « Emplacement » =
*À droite de la porte d'entrée, sous la boîte aux lettres*, « Code boîte à clés » = *4821*,
« Instructions », « Digicode » = *A17B*, « Interphone » = *Appartement 3, 2e étage*,
« Parking » = *Place n° 8 au sous-sol, badge sur le trousseau.*, bouton « Ouvrir dans Maps »,
puis deux étapes : *Garez-vous au sous-sol* (pastille « Parking ») et *Entrez dans l'immeuble*
(pastille « Porte »).

### Carte avant l'arrivée — *Aperçu*

Volontairement minuscule et **non cliquable**. Icône voiture, titre « Accès & parking », et
**une seule ligne** : le moyen d'accès. Aperçu lu : *« Boîte à clés »*.

---

## Appareils

> *À quoi sert la surface :* la liste des appareils (four, lave-linge, climatisation…) avec,
> pour chacun, la pièce, les consignes d'utilisation et le lien vers le manuel, ainsi qu'une
> consigne de sécurité commune.

**3 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **prise électrique**, titre « Appareils ». Elle ouvre la liste complète **en
plein écran**.

Elle ne montre **que les appareils mis en avant par l'hôte, et actifs — cinq au maximum**.
Chaque appareil est une ligne de liste : **emoji** à gauche (celui saisi par l'hôte, absent si
non renseigné), **nom**, **pièce** en sous-titre (absente si non renseignée), **chevron** à
droite. Chaque ligne mène à la fiche de l'appareil.

Si aucun appareil n'est mis en avant, un bloc d'état vide **à l'intérieur de la carte** :
icône prise, « **Rien en avant pour l'instant** », « Ouvrez la liste pour voir tous les
appareils disponibles. »

### Détail — *Aperçu* — la liste complète

Le corps est **une seule carte surélevée** contenant toutes les lignes. Mêmes lignes que
ci-dessus, mais **tous** les appareils actifs, pas seulement ceux mis en avant.

Aperçu lu : *🍳 Plaques à induction / Cuisine*, *🌀 Lave-linge / Salle de bain*,
*📺 Télévision / Salon*.

Si la liste est vide : bloc d'état vide dans la carte, « **Aucun appareil** » / « Aucun guide
appareil n'est disponible pour ce séjour. »

### Fiche d'un appareil — *Aperçu*

Une page par appareil, atteinte depuis n'importe quelle ligne.

1. **En-tête** : l'emoji en très gros à gauche, puis en colonne le **nom** en très gros et la
   **pièce** en petit et atténué.
2. Une **carte surélevée** « **Mode d'emploi** » (en surtitre). Deux formes possibles :
   - si l'hôte a écrit des **étapes**, chaque étape devient une ligne numérotée (le numéro en
     titre, le texte en sous-titre) ;
   - sinon, le texte est rendu **en bloc de texte riche**, tel que l'hôte l'a mis en forme.
     Aperçu lu : *« Appuyez 2 secondes sur la touche marche, puis choisissez le foyer. »* /
     *« Seules les casseroles à fond aimanté chauffent. »*
3. Un **bandeau d'information sans titre** avec la consigne de sécurité de l'appareil. Aperçu
   lu : *« La surface reste chaude quelques minutes après l'arrêt. »* Absent si non renseignée.
4. Un lien « **Notice / manuel** », vers le document de l'hôte. Absent si non renseigné.
5. Un bouton de contour « **Un souci avec cet appareil ?** », qui ouvre la messagerie de
   l'hôte en lui transmettant le nom de l'appareil.

Si l'appareil demandé n'existe plus ou a été retiré : bloc d'état vide seul,
« **Appareil introuvable** » / « Ce guide n'est plus disponible ou a été retiré. »

**Ce qui varie :** le mode d'emploi a deux mises en page très différentes (étapes numérotées
ou texte libre). L'emoji, la pièce, la consigne de sécurité et le manuel sont tous
indépendamment absents ou présents.

---

## Bienvenue *(module « sections »)*

> *À quoi sert la surface :* un aperçu des premières sections sur l'accueil du livret, puis le
> texte complet de chacune, avec titres, listes et citations.

**2 surfaces. Aucune n'a d'aperçu — tout vient du code.**

### Carte d'accueil — *Code*

Carte à icône **maison**, titre « Bienvenue ». Elle ouvre le texte complet dans une **feuille
qui remonte du bas**.

Elle montre **les deux premières sections** non vides, dans l'ordre de l'hôte. Entre elles,
un **séparateur**.

Une règle de mise en page à retenir : **si la carte ne contient qu'une seule section, son
titre n'est pas répété** — l'en-tête de la carte le dit déjà. S'il y en a plusieurs, chaque
section reçoit son **titre en petit** au-dessus de son corps.

Le corps de chaque section est du **texte riche mis en forme par l'hôte** — titres
intermédiaires, listes, citations. Deux formats de saisie existent côté hôte ; à l'écran, le
rendu est le même.

Si rien n'est écrit : le bloc d'état vide commun, « **Rien pour le moment** » / « Votre hôte
n'a encore rien ajouté ici. »

### Feuille de détail — *Code*

Exactement les mêmes blocs, mais **toutes** les sections au lieu de deux, séparées par des
séparateurs, avec un espacement plus large. Le titre de la feuille est posé par le livret.

**Ce qui varie :** le nombre de sections, la longueur de chaque texte (elle est libre), et la
présence ou non des titres de section selon qu'il y en a une ou plusieurs.

---

## Bons plans *(local-guide)*

> *À quoi sert la surface :* jusqu'à six adresses choisies par l'hôte (catégorie, distance,
> description), une carte des lieux et du logement et, si l'hôte les active, des activités et
> billets réservables autour du logement, avec photo, prix et note.

**3 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **épingle**, titre « Bons plans ». Ouvre le détail en **feuille du bas**.

1. Un **bandeau d'information** titré « **Info** », avec l'avertissement de l'hôte. Aperçu du
   détail lu : *« Suggestions de votre hôte, sans partenariat. »*
2. Chaque adresse, en ligne de liste : **nom** en titre, puis en sous-titre la **catégorie** et
   la **distance** séparées par un point médian (*« Boulangerie · 350 m »*) — chacune des deux
   peut manquer. Si l'hôte a posé une **étiquette**, elle apparaît en **pastille** sous le
   titre (*« Samedi matin »*).
   Si l'adresse porte un lien, **la ligne entière devient cliquable** et part vers ce lien.
3. **Pas de carte géographique** dans la carte d'accueil.
4. La section **billetterie**, si elle est active : titre « **Billets à proximité** », puis les
   billets **sans image**, en lignes cliquables, avec en sous-titre le prix et la note
   (« Dès 22 € · ★ 4,6 (18 234 avis) »). **Trois billets au maximum**, pour rester un aperçu.
   Puis les deux mentions obligatoires (voir *États communs §8*).
5. La section **activités**, si elle est active : titre « **Activités & billets** », un texte
   d'introduction facultatif, un lien « Voir les activités à <ville> » (ou « Voir les
   activités » si la ville n'est pas lisible), les liens choisis par l'hôte (libellé de l'hôte,
   ou « Ouvrir l'activité » à défaut), puis la mention d'affiliation.

### Détail — *Aperçu* — feuille du bas

Le même corps, enrichi :

1. **En tête, la carte géographique.** Les adresses situées en repères de point d'intérêt, plus
   le logement en repère de logement. Aperçu lu : quatre repères — *La boulangerie du village*,
   *Marché provençal*, *Lac de Saint-Cassien*, et *Villa Azur* pour le logement.
2. Le bandeau « Info ».
3. Chaque adresse, enrichie : sous le titre et le sous-titre viennent la **note** de l'hôte en
   petit, la **description** en texte courant, puis un lien explicite « **Ouvrir le lien** ».
   La ligne elle-même n'est plus cliquable.
   Aperçu lu : *La boulangerie du village* / *Boulangerie · 350 m* / *« Croissants dès 7 h,
   fermée le lundi. »* ; *Marché provençal* / *Marché · 1,2 km* / pastille *Samedi matin* /
   *« Fruits, fromages et fleurs, le samedi matin. »* ; *Lac de Saint-Cassien* / *Baignade ·
   9 km* / *« Eau calme et plage de galets, idéal avec des enfants. »*
4. La billetterie **avec les images** cette fois (format 16/9), l'accroche, le **crédit de la
   photo** obligatoire sous l'image, et un lien « **Réserver sur Tiqets** ». Tous les billets,
   plus de plafond.

### Carte avant l'arrivée — *Aperçu*

Minuscule, **non cliquable**. Icône épingle, titre « Bons plans », **une ligne** : le nombre
d'adresses. Aperçu lu : *« 3 adresses »*. Si aucune adresse, la ligne disparaît et il ne reste
que le titre.

**Ce qui varie :** la carte disparaît si aucune adresse n'est située. Les sections activités et
billetterie sont chacune activables séparément — la surface peut n'être qu'une liste
d'adresses. Les images de billets peuvent manquer.

---

## Checklists

> *À quoi sert la surface :* une carte avec les listes voyageur à cocher, au moment choisi par
> l'hôte ; la liste de départ reste sur l'écran de fin de séjour.

**2 surfaces. Aucune n'a d'aperçu — tout vient du code.**

### Carte d'accueil — *Code*

Carte à icône **liste cochée**. Elle **ne s'ouvre pas** : tout se fait dedans.

Son en-tête est particulier — **le titre est une date**, pas le nom du module : « **Départ
mardi à 11:00** », construit à partir de l'heure de départ du séjour dans le fuseau du
logement. Les sept jours existent en français (« lundi », « mardi »…). **Quand le séjour ne
porte pas encore d'heure de départ, le titre retombe sur « Checklists ».**
Sous le titre, en sous-titre : « **Voir la checklist** ».

Contenu :

1. Une ligne d'**avancement** en petit, de la forme `3 / 8 — 37%`.
2. Puis, pour chaque liste : **son nom en titre** — mais **seulement s'il y a plusieurs
   listes** ; avec une seule liste, le nom est omis.
3. Puis chaque élément, en **ligne cochable**, avec son libellé et son état. **Un appui coche
   ou décoche directement, sur place** — aucune page ne s'ouvre. C'est la seule surface du
   livret à modifier son propre contenu sous le doigt du voyageur : l'avancement en tête doit
   suivre.

Deux états remplacent tout le contenu, **en gardant la carte et son en-tête** :

- « **Rien à cocher pour ce séjour.** » — aucune liste applicable. Icône liste cochée.
- « **La checklist s'ouvrira au moment choisi par l'hôte.** » — pas encore l'heure. **L'icône
  change : horloge.**

Dans ces deux cas, le titre redevient « Checklists ».

### Carte de fin de séjour — *Code*

Exactement la même carte, avec une seule différence : **elle ne montre que les listes de
départ**. Elle reste cochable une fois le séjour terminé.

**Ce qui varie :** le nombre de listes (et donc la présence des titres de liste), le nombre
d'éléments, l'avancement, et le titre-date qui peut manquer.

---

## Consommables

> *À quoi sert la surface :* un formulaire pour signaler qu'un consommable manque ou sera
> bientôt vide, avec une précision facultative, puis la liste des signalements envoyés pendant
> le séjour.

**2 surfaces.**

### Carte d'accueil — *Aperçu*

Carte à icône **colis**, titre « Consommables ». La carte entière **et** sa dernière ligne
ouvrent le formulaire en **feuille du bas**.

Deux états de contenu :

- **Aucun signalement encore.** Une phrase : « **Il manque du savon, du papier, du café… ?
  Prévenez votre hôte.** »
- **Au moins un signalement.** À la place : « **Merci — votre hôte a été prévenu.** » en texte
  courant, puis « **Vos signalements ce séjour** » en petit, puis **une ligne par signalement**
  — le nom du consommable en titre, et en sous-titre « **Il n'y en a plus** » ou « **Bientôt
  vide** ».

Dans les deux cas, en dernier : une ligne de liste « **Signaler un manque** », icône colis à
gauche, **chevron** à droite.

Si le catalogue de consommables est vide, la carte garde son en-tête et affiche une seule
ligne : « **Pour signaler un manque, contactez directement votre hôte.** » — sans bouton.

### Formulaire — *Aperçu* — feuille du bas

Un formulaire nu, sans cadre de carte.

1. Champ « **Consommable** », obligatoire. Une **liste de choix compacte**, une entrée par
   consommable du catalogue, chacune avec l'icône colis. Aperçu lu, dans l'ordre : *Papier
   toilette*, *Savon*, *Gel douche*, *Shampoing*, *Café*, *Tablettes lave-vaisselle*,
   *Essuie-tout*, *Lessive*. **Le premier est présélectionné.**
2. Champ « **Situation** », obligatoire. Deux choix : « **Il n'y en a plus** » (icône cercle
   barré) et « **Bientôt vide** » (icône jauge). Le premier est présélectionné.
3. Champ « **Précision (optionnel)** ». Zone de texte, avec pour texte d'aide : *« Ex. salle de
   bain du haut, dernière tablette… »*
4. Bouton plein « **Prévenir l'hôte** ».

**Ce qui varie :** le nombre d'entrées du catalogue — de deux à quinze, la liste de choix doit
tenir dans les deux cas. Le nombre de signalements déjà envoyés.

---

## Événements

> *À quoi sert la surface :* les prochains événements autour du logement, trouvés
> automatiquement et complétés par la sélection de l'hôte, avec une carte quand les lieux sont
> connus.

**3 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **calendrier**, titre « Événements ». Ouvre le détail en **feuille du bas**.

1. Un **bandeau d'information** titré « **Info** », avec l'avertissement. Aperçu du détail
   lu : *« Programme indicatif, à vérifier auprès des organisateurs. »*
2. Chaque événement, en ligne de liste. **Devant**, un petit **badge de date** de la forme
   « mar. 02 ». En titre, le nom ; en sous-titre, **le lieu et l'heure joints par un point
   médian** : *« Place du village · à 18:00 »* — chacun des deux peut manquer.
   **Quand la date est inconnue**, le badge de date disparaît et une **pastille « Date à
   confirmer »** la remplace sous le titre. Cette pastille **n'existe que dans la carte
   d'accueil**, pas dans le détail.
   Si l'événement porte un lien, **la ligne entière devient cliquable**.
3. **Pas de carte géographique** dans la carte d'accueil.

### Détail — *Aperçu* — feuille du bas

1. **En tête, la carte géographique**, si l'hôte l'a activée et qu'au moins un lieu est situé.
   Aperçu lu : un repère, *Marché nocturne des artisans*.
2. Le bandeau « Info ».
3. Chaque événement enrichi : badge de date, titre, sous-titre, puis la **note** de l'hôte en
   petit, puis un lien « **Ouvrir le lien** ».
   Aperçu lu : *mar. 02 — Marché nocturne des artisans / Place du village · à 18:00 / « Entrée
   libre »* ; *jeu. 04 — Concert de jazz en plein air / Jardin public · à 20:30 / lien* ;
   *dim. 07 — Vide-grenier du dimanche / Parking de la plage · à 08:00*.

### Carte avant l'arrivée — *Aperçu*

Minuscule, mais **cliquable** (elle ouvre le détail en feuille du bas — c'est la seule carte
avant l'arrivée du livret à être cliquable, avec celle de Horaires trains). Icône calendrier,
titre « Événements », **une ligne** : le prochain événement, nom et date joints par un point
médian. Aperçu lu : *« Marché nocturne des artisans · à 18:00 »*.

S'il n'y a rien à montrer, la ligne disparaît et il ne reste que le titre.

---

## Horaires & accès *(facility-hours)*

> *À quoi sert la surface :* les horaires de chaque équipement partagé et une note générale.

**2 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **horloge**, titre « Horaires & accès ». Ouvre le détail en **feuille du bas**.

1. Un **bandeau d'information** titré « **À savoir** », avec la note générale, s'il y en a une.
2. Puis un équipement par ligne, sous forme de **paire libellé / valeur** : à gauche le nom de
   l'équipement, à droite ses horaires. Quand l'hôte n'a pas saisi de ligne d'horaires unique,
   les lignes détaillées sont **jointes par des points médians** pour en fabriquer une.

### Détail — *Aperçu* — feuille du bas

Le même contenu, mais chaque équipement devient une **ligne de liste** au lieu d'une paire :
le nom en titre, les horaires en sous-titre, puis **chaque ligne détaillée en petit** sous la
ligne, puis la **note** de l'équipement en petit.

Aperçu lu : bandeau « À savoir » / *« Horaires susceptibles de varier les jours fériés. »* ;
*Piscine — 9 h – 20 h* / *Tous les jours, de juin à septembre* / *Enfants sous la surveillance
d'un adulte* / *Bonnet de bain non obligatoire.* ; *Laverie — 7 h – 22 h* / *Rez-de-chaussée,
bâtiment B* ; *Accueil de la résidence — 8 h 30 – 12 h · 14 h – 18 h* / *Du lundi au samedi*.

**Ce qui varie :** c'est le module où l'écart aperçu/détail change **la primitive elle-même**.
À dessiner comme deux composants. Le nombre de lignes détaillées par équipement est libre.

---

## Horaires trains *(train)*

> Ce module n'a pas de phrase descriptive côté voyageur dans sa fiche — à établir.

**3 surfaces.** Attention : ce module est à un stade préliminaire. Sa gare, ses destinations et
ses horaires **sont figés dans le code**, pas configurés par l'hôte — toutes les valeurs
ci-dessous sont donc à la fois des exemples et le contenu réel actuel.

### Carte d'accueil — *Code*

Carte à icône **train**, titre « Horaires trains ». Ouvre le détail **en plein écran**.

1. Une ligne de **légende atténuée** : la gare et sa distance, jointes par un point médian —
   « **Gare d'Antibes · 2,3 km** ».
2. Puis **quatre prochains départs**, toutes destinations confondues, en **entrées horodatées** :
   l'**heure** mise en avant, la **destination** en titre, le **quai** en sous-titre.

### Détail — *Aperçu* — plein écran

1. Une **carte surélevée** avec deux paires libellé / valeur : « **Départ** » = *Gare
   d'Antibes · 2,3 km*, « **Destination** » = *Nice-Ville*.
2. Une **barre de filtres** : une pastille filtrante par destination, la sélectionnée en
   évidence. Aperçu lu : *Nice-Ville* (sélectionnée), *Cannes*, *Monaco*, *Grasse*. Chaque
   pastille recharge la page pour cette destination.
3. Un titre « **Prochains départs** ».
4. Une **carte surélevée** contenant les entrées horodatées de la destination choisie. Aperçu
   lu : *08:12 — Nice-Ville — quai 2 · direct* ; *08:42 — Nice-Ville — quai 1 · direct* ;
   *09:05 — Nice-Ville — quai 2 · 1 arrêt* ; *09:38 — Nice-Ville — quai 1 · direct*.
5. Une **légende atténuée** en pied : « **Horaires indicatifs · TER SUD PACA** ».

### Carte avant l'arrivée — *Aperçu*

Minuscule et **cliquable** (ouvre le détail en plein écran). Icône train, titre « Horaires
trains », **une ligne atténuée** : le prochain départ. Aperçu lu : *« Gare d'Antibes →
Nice-Ville · 08:12 »*. Sans départ au tableau, la ligne se réduit au nom de la gare.

---

## Objets perdus ou trouvés *(lost-found)*

> *À quoi sert la surface :* un formulaire pour déclarer un objet perdu ou trouvé, pendant ou
> après le séjour, avec le message éventuel de l'hôte, puis la liste des déclarations envoyées.

**3 surfaces.**

### Carte d'accueil — *Aperçu*

Carte à icône **loupe**, titre « **Objets perdus ou trouvés** ». La carte entière et sa
dernière ligne ouvrent le formulaire en **feuille du bas**.

- **Aucune déclaration encore :** une phrase — « **Vous avez perdu quelque chose ou trouvé un
  objet ? Prévenez votre hôte — il vous recontactera.** »
- **Au moins une :** « **Merci — votre hôte a été prévenu.** », puis « **Vos signalements ce
  séjour** » en petit, puis **une ligne par déclaration** — la description de l'objet en titre
  (en texte brut, même si l'hôte l'a mise en forme), et en sous-titre « **J'ai perdu quelque
  chose** » ou « **J'ai trouvé quelque chose** ».
  Aperçu lu : *Chargeur de téléphone blanc* / *J'ai perdu quelque chose*.

En dernier : ligne de liste « **Nouvelle déclaration** », icône loupe, chevron.

> **Écart repéré :** la fiche du module promet « le message éventuel de l'hôte » sur cette
> surface. **Ce message n'existe nulle part dans le code de la surface voyageur.** Rien à
> dessiner ici tant que ce n'est pas tranché.

### Formulaire — *Aperçu* — feuille du bas

1. Champ « **Type** », obligatoire. Deux choix, liste compacte : « **J'ai perdu quelque
   chose** » (icône loupe barrée) et « **J'ai trouvé quelque chose** » (icône colis-loupe).
   **Aucun n'est présélectionné** — à la différence du formulaire Consommables.
2. Champ « **De quoi s'agit-il ?** », obligatoire. Champ texte, aide : *« ex. Écharpe bleue,
   clé de chambre »*
3. Champ « **Comment vous joindre (optionnel)** ». Champ texte, aide : *« Téléphone ou
   e-mail »*
4. Champ « **Détails** ». Zone de texte, aide : *« Optionnel — où, quand, couleur, marque… »*
5. Bouton plein « **Envoyer** ».

### Carte de fin de séjour — *Aperçu*

**Strictement identique** à la carte d'accueil, à la surface près. Même titre, même contenu,
même formulaire. Un seul dessin pour les deux.

---

## Pré-arrivée

> *À quoi sert la surface :* un court formulaire dans le livret, selon les questions choisies
> par l'hôte : heure d'arrivée estimée, occasion, allergies, nombre de voyageurs, besoins
> particuliers, pièce d'identité et message pour l'hôte. Un e-mail prévient le voyageur quand
> le formulaire est disponible.

**2 surfaces.** C'est la surface la plus conditionnelle du livret.

### Carte d'accueil — *Aperçu (état « en attente ») + Code (les deux autres états)*

Une carte **teintée**, titre « **Avant votre arrivée** », avec un **sous-titre** et une
**icône** qui changent selon l'état :

| État | Icône | Teinte | Sous-titre |
|---|---|---|---|
| Pas encore ouvert | horloge | primaire | « Formalités avant votre séjour » |
| À remplir | horloge | primaire | « Formalités avant votre séjour » |
| Rempli | coche | succès | « Tout est prêt pour votre séjour. » |

Contenu, sans espacement entre les lignes (elles se collent comme une petite liste de tâches) :

1. **La tâche « fiche de police »**, qui **n'appartient pas à ce module** : c'est un fragment
   posé par la plateforme, rendu en ligne de tâche. **Le livret le montre ou l'omet selon que
   la déclaration est obligatoire pour ce logement.** À dessiner comme une ligne de tâche
   générique dont le module ne connaît pas le contenu.
2. **La tâche « formulaire de pré-arrivée »**, une ligne de liste à icône presse-papiers avec
   chevron, qui ouvre le formulaire **en plein écran** :
   - *à remplir* : titre « **Formulaire de pré-arrivée** », sous-titre « **Heure d'arrivée,
     occasion, allergies** » ;
   - *rempli* : même titre, sous-titre « **Complété** » — toujours cliquable, pour relire ou
     modifier ;
   - *pas encore ouvert* : **la ligne disparaît entièrement**. Pas de texte « bientôt
     disponible ». La carte peut alors ne contenir que la tâche police — et si celle-ci n'est
     pas requise non plus, **le livret masque la carte**.

### Formulaire — *Aperçu* — plein écran

Formulaire nu. **Chaque champ est indépendamment activable par l'hôte** : la mise en page doit
tenir avec un seul champ comme avec sept.

1. Une phrase d'introduction : « **Aidez-nous à préparer votre venue en 1 minute. Ces infos ne
   servent qu'à votre accueil.** »
2. « **Heure d'arrivée estimée** », **obligatoire** — un sélecteur d'heure.
3. « **Occasion spéciale ?** » — champ texte, aide *« Anniversaire (optionnel) »*
4. « **Allergies / régime alimentaire** » — champ texte, aide *« Optionnel »*
5. « **Nombre de voyageurs** » — champ texte, aide *« Ex. 2 adultes, 1 enfant »*
6. « **Besoins particuliers** » — champ texte, aide *« Lit bébé, PMR… (optionnel) »*
7. « **Pièce d'identité** » — champ texte, aide *« Nom tel qu'indiqué sur la pièce (fiche
   police) »*
8. « **Un message pour l'hôte ?** » — zone de texte, aide *« Ex. nous arriverons peut-être un
   peu en retard… »*. **Toujours présent**, ce n'est pas une question optionnelle.
9. Bouton plein : « **Envoyer** », ou « **Mettre à jour** » si le formulaire a déjà été envoyé.

**Quand le voyageur revient**, les champs sont **pré-remplis** avec ses réponses.

### Le formulaire après le check-in — *Code*

Après l'arrivée, le formulaire n'est plus modifiable et **change complètement de forme** —
ce n'est plus un formulaire mais un récapitulatif :

1. « **Merci, c'est noté.** » en texte courant.
2. « **Le formulaire n'est plus modifiable après le check-in.** » en petit.
3. Puis **une ligne de liste par réponse**, non cliquable, **sans chevron** : le libellé de la
   question en titre, la réponse en sous-titre, et une icône à gauche propre à chaque question
   — horloge (heure d'arrivée), étoile (occasion), triangle d'alerte (allergies), personnes
   (nombre de voyageurs), maison (besoins particuliers), presse-papiers (pièce d'identité),
   message (message à l'hôte).
   **Une réponse laissée vide s'affiche comme un tiret cadratin (—)**, pas comme une ligne
   absente — sauf le message à l'hôte, dont la ligne disparaît s'il est vide.

**À dessiner : c'est un troisième écran**, pas une variante du formulaire.

---

## Règlement intérieur *(rules)*

> *À quoi sert la surface :* les règles de la maison sous forme de courtes consignes illustrées
> d'une icône (calme, animaux, bruit…), en aperçu puis en liste complète.

**2 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **balance**, titre « Règlement intérieur ». Ouvre la liste complète **en plein
écran**.

Elle montre **les quatre premières règles** ayant un titre. Chaque règle est une ligne de
liste : **une icône à gauche**, le **titre** de la règle, et un **sous-titre** facultatif.
L'icône est celle choisie par l'hôte ; **à défaut, une coche dans un cercle**.

Si aucune règle n'a de titre : une seule ligne de texte courant, « **L'hôte n'a pas encore
publié de règles pour ce logement.** » — à l'intérieur de la carte.

Si le module n'a **aucune** règle du tout, la surface entière est remplacée par le bloc d'état
vide commun : icône balance, « **Aucun règlement pour le moment** » / « L'hôte n'a pas encore
publié de règles pour ce logement. »

### Détail — *Aperçu* — plein écran

Le corps est **une seule carte surélevée** contenant **toutes** les règles, collées les unes
aux autres sans espacement.

Aperçu lu, dans l'ordre :
*Calme après 22 h* / *Merci de penser au voisinage* — icône horloge ;
*Logement non-fumeur* / *Vous pouvez fumer sur la terrasse* — icône croix ;
*Pas de fête ni d'événement* — icône personnes, **sans sous-titre** ;
*Animaux bienvenus* / *Prévenez-nous avant votre arrivée* — icône coche dans un cercle ;
*Départ avant 10 h* / *Laissez les clés sur la table de la cuisine* — icône horloge.

**Ce qui varie :** le sous-titre peut manquer sur n'importe quelle ligne. Le nombre de règles
est libre — la carte d'accueil en montre quatre, la page toutes. Les icônes viennent d'un jeu
ouvert (l'hôte les choisit), avec une substitution silencieuse pour deux ou trois cas — la
ligne à dessiner doit accepter **n'importe quelle icône**.

---

## Signaler un problème *(issue-report)*

> *À quoi sert la surface :* un formulaire pour signaler un problème par catégorie (appareil,
> propreté, bruit, accès…), avec un résumé, des détails et une photo, puis la liste des
> signalements du séjour.

**2 surfaces.**

### Carte d'accueil — *Aperçu*

Carte à icône **triangle d'alerte**, titre « **Signaler un problème** ». Ouvre le formulaire en
**feuille du bas**.

Même structure que Consommables et Objets perdus :

- **Aucun signalement :** « **Dites-nous ce qui ne va pas — nous transmettrons à votre
  hôte.** »
- **Au moins un :** « **Merci — votre hôte a été prévenu.** », puis « **Vos signalements ce
  séjour** » en petit, puis une ligne par signalement — **le résumé** en titre, **la catégorie**
  en sous-titre. Aperçu lu : *Le four ne chauffe plus* / *Appareil ou équipement*.

En dernier : ligne « **Nouveau signalement** », icône triangle d'alerte, chevron.

### Formulaire — *Aperçu* — feuille du bas

1. Champ « **Catégorie** », obligatoire. Liste de choix compacte, **cinq entrées, aucune
   présélectionnée** : « **Appareil ou équipement** » (icône prise), « **Propreté** » (icône
   étincelles), « **Bruit ou nuisance** » (icône haut-parleur), « **Accès ou clés** » (icône
   clé), « **Autre** » (icône bulle).
2. Champ « **Résumé court** », obligatoire. Champ texte, aide : *« ex. Lave-vaisselle ne se
   vide pas »*
3. Champ « **Détails** ». Zone de texte, aide : *« Optionnel — quoi, quand, où »*
4. Champ « **Photo** ». **Un envoi d'image** — c'est le seul champ de ce genre dans tout le
   livret voyageur. À dessiner : état vide, état en cours d'envoi, état avec image.
5. Bouton plein « **Envoyer** ».

---

## Tri & déchets *(waste-recycling)*

> *À quoi sert la surface :* chaque bac avec sa couleur et ce qu'on y jette, et les jours de
> collecte.

**2 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **recyclage**, titre « Tri & déchets ». Ouvre le détail en **feuille du bas**.

Chaque bac occupe **une ligne à pastille de couleur** : un point coloré à gauche, puis un
libellé unique qui joint **le nom du bac et le contenu**, séparés par un tiret cadratin. Le
contenu est la liste des matières, **jointes par des virgules**.

**Si la couleur du bac n'est pas reconnue**, la ligne bascule sur une **ligne de liste
ordinaire** : nom en titre, matières en sous-titre. Deux formes de ligne à dessiner, donc.

En pied, si un calendrier de collecte est renseigné : un **bandeau d'information** titré
« **Collecte** ».

### Détail — *Aperçu* — feuille du bas

Même chose, à une nuance près : **pour les bacs sans couleur reconnue**, chaque matière
devient **sa propre ligne de texte en petit** sous le titre, au lieu d'être jointe par des
virgules. Les bacs à pastille de couleur, eux, sont identiques dans les deux surfaces.

Aperçu lu — quatre pastilles :
*jaune* — **Bac jaune — Emballages plastique et métal, Cartons et papiers** ;
*verte* — **Colonne à verre — Bouteilles et bocaux, sans bouchon** ;
*marron* — **Bac à biodéchets — Épluchures, marc de café, restes de repas** ;
*grise* — **Ordures ménagères — Tout le reste, en sac fermé**.
Puis le bandeau « **Collecte** » : *« Bacs à sortir la veille au soir : mardi pour le jaune,
vendredi pour les ordures ménagères. »*

---

## Urgences & utiles *(emergency-contacts)*

> *À quoi sert la surface :* les numéros utiles (pharmacie, médecin…), le rappel du 112 et, si
> l'hôte le souhaite, son téléphone direct.

**2 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **téléphone**, titre « Urgences & utiles ». Ouvre le détail en **feuille du bas**.

**Le bandeau d'urgence et le lien 112 n'y sont pas** — c'est la seule différence avec le détail.

1. Si l'hôte a donné son numéro : une **ligne entièrement cliquable** — titre « **Votre
   hôte** », sous-titre le numéro, et à droite l'action « **Appeler** ».
2. Puis un contact par ligne, sur le même modèle : le **nom** en titre, le **numéro** en
   sous-titre, « **Appeler** » à droite. Si l'hôte a rangé le contact dans une catégorie, elle
   apparaît **en tête de ligne**. Si le contact porte une note, elle s'affiche **en petit** sous
   la ligne.

Chaque ligne déclenche l'appel directement.

### Détail — *Aperçu* — feuille du bas

Le même corps, **encadré** :

1. **En tête**, un bandeau d'information titré « **Urgence vitale** » : « **En cas d'urgence
   vitale, composez le 112 ou le 15 (SAMU) selon les consignes locales.** »
2. Les contacts, comme ci-dessus.
3. **En pied**, un lien « **Composer le 112** ».

Aperçu lu : bandeau, puis *Votre hôte / 06 39 98 12 34 / Appeler* ; *Pharmacie de garde / 3237
/ Appeler* avec la note *« Service national »* ; *Médecin généraliste / 01 99 00 12 34 /
Appeler* avec la note *« Cabinet du centre, sur rendez-vous »* ; *SAMU / 15 / Appeler* ; puis
le lien « Composer le 112 ».

**Ce qui varie :** la ligne « Votre hôte » peut manquer. La catégorie et la note de chaque
contact sont indépendamment facultatives. L'état vide de ce module est le seul à donner **une
information utile** plutôt qu'une promesse : « En cas d'urgence vitale, composez le 112. »

---

## Parking VE *(ev-parking)*

> *À quoi sert la surface :* la place réservée, le code barrière et le code de la borne de
> recharge, révélés selon le calendrier de l'hôte, avec un lien vers la carte et des consignes.
> La place est aussi rappelée dans les e-mails d'arrivée.

**2 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **éclair**, titre « Parking VE ». Ouvre le détail en **feuille du bas**.
**Le contenu est rigoureusement identique à celui du détail** — ce module n'a pas de version
condensée.

### Détail — *Aperçu* — feuille du bas

1. Le **bandeau des codes non révélés**, si des codes existent et ne sont pas encore visibles
   (voir *États communs §5*).
2. « **Votre place** » + le texte de l'hôte. Aperçu lu : *Place n° 8, niveau -1*.
3. Un lien « **Ouvrir la carte** », vers le plan du parking.
4. « **Code barrière** » en chasse fixe, puis un bouton de contour « **Copier le code
   barrière** » (toast : « Code barrière copié »). Aperçu lu : *1357*.
5. « **PIN borne** » en chasse fixe, puis un bouton de contour « **Copier le PIN borne** »
   (toast : « PIN borne copié »). Aperçu lu : *2468*.
6. Les **consignes** de l'hôte, en petit. Aperçu lu : *« Branchez le câble Type 2 fourni, puis
   saisissez le PIN sur la borne. »*

**Ce qui varie :** chacun des six blocs peut manquer indépendamment. Tant que les codes ne sont
pas révélés, les deux boutons de copie **n'existent pas** et les valeurs sont masquées.

---

## Votre avis *(guest-reviews)*

> *À quoi sert la surface :* après le séjour, un message de remerciement, un bouton et un QR
> code vers la page d'avis Airbnb, et/ou une notation par étoiles avec commentaire directement
> dans le livret.

**2 surfaces.**

### Carte d'accueil — *Code* (même contenu que la carte de fin de séjour, ci-dessous)

Carte à icône **étoile**, titre « Votre avis ». **Elle ne s'ouvre pas** : tout se fait dedans.

**Avant l'arrivée**, la carte garde son en-tête et affiche une seule ligne : « **Vous pourrez
laisser votre avis dès votre arrivée.** » Après l'arrivée, elle bascule sur le contenu complet.

### Carte de fin de séjour — *Aperçu*

Le contenu complet, sur la même carte :

1. Une question en **titre**, avec le nom du logement : « **Comment s'est passé votre séjour à
   Villa Azur ?** »
2. Le **remerciement** de l'hôte, en texte courant. S'il n'en a pas écrit, un texte par défaut
   prend le relais : « **On espère que vous avez aimé votre séjour autant que nous. Si vous
   avez un instant, un avis fait toute la différence.** » Aperçu lu (texte d'hôte) : *« Merci
   pour votre séjour ! Votre avis aide les prochains voyageurs à nous choisir. »*
3. **Si l'hôte a activé Airbnb** (et que le lien existe) : un bouton plein « **Laisser un avis
   sur Airbnb** ». Et s'il a aussi activé le QR code : un **QR code de 144 points** sous le
   bouton, puis la légende « **Ou scannez le QR vers la page d'avis Airbnb** ».
4. **Si l'hôte a activé l'avis dans le livret**, et **seulement si Airbnb est aussi activé** :
   une légende de bascule, « **Ou sur Portaki** ». Puis un formulaire :
   - champ « **Votre note** » — une **liste déroulante** de cinq entrées, ★ à ★★★★★,
     **présélectionnée sur cinq étoiles** ;
   - champ « **Un mot sur votre séjour** » — zone de texte, aide *« Un mot sur votre séjour… »* ;
   - bouton plein « **Envoyer mon avis** ».

**Ce qui varie :** trois configurations possibles — Airbnb seul, livret seul, les deux. Le QR
code n'apparaît qu'avec Airbnb. La légende « Ou sur Portaki » n'apparaît **que** quand les deux
coexistent.

> **À noter pour le dessin :** la note est une **liste déroulante d'étoiles**, pas une rangée
> d'étoiles cliquables. C'est ce que fait le code, et l'aperçu le confirme. Si l'intention est
> une rangée cliquable, c'est un écart à signaler avant de dessiner.

---

## Météo

> *À quoi sert la surface :* la température et le temps actuels, avec l'indice UV quand il est
> disponible, et les prévisions sur 5 jours. Un résumé figure aussi dans l'e-mail du jour
> d'arrivée.

**3 surfaces.**

### Carte d'accueil — *Code*

**La seule carte du livret dont le titre est une donnée vivante.** Icône : **celle qui
correspond au temps qu'il fait** (soleil, nuages, pluie…). Titre : « **23°C · Ensoleillé** ».
Sous-titre : « **La météo des prochains jours** ». Ouvre les prévisions en **feuille du bas**.

Contenu :

1. Un **bloc météo en grand** : à gauche, **l'icône du temps en 56 points** ; à droite, en
   colonne, la **température en très gros** (teintée selon sa valeur), puis la **description**
   en petit, puis la **ville** en petit.
2. Un **séparateur**.
3. Une **bande de prévisions sur cinq colonnes**, une par jour : le jour abrégé
   (« Lun », « Mar »…), l'icône du temps en 28 points, et une température en petit et en gras,
   teintée.

**La carte d'accueil ne montre pas l'indice UV** — c'est une décision explicite.

### Feuille des prévisions — *Aperçu* — feuille du bas

1. Le même **bloc météo en grand**. Aperçu lu : icône soleil, *23 °C* teinté avertissement,
   *Ensoleillé*, *Cannes*.
2. Un **séparateur**.
3. Une **grille de trois cases sur deux colonnes** — les conditions actuelles en détail. Chaque
   case : une petite icône (14 points) + un libellé en gras et en petit, puis la valeur en
   petit dessous. Aperçu lu : *goutte — Humidité — 58%* ; *soleil — UV — UV élevé* ;
   *vent — Vent — 12 km/h*.
   L'échelle d'UV a quatre libellés : « UV faible », « UV modéré », « UV élevé », « UV
   extrême ».
4. Un **séparateur**.
5. Un titre « **Prévisions sur 5 jours** ».
6. Un **tableau à sept colonnes**. Sa ligne d'en-tête : « **Jour** », une colonne sans libellé
   (l'icône), « **Min** », « **Max** », puis trois en-têtes à icône — *nuage-pluie* + « **Pluie** »,
   *goutte* + « **Hum.** », *vent* + « **Vent** ».
   Puis cinq lignes. Aperçu lu, ligne à ligne :
   *Lun 1 — soleil — 17° — 25° — 0% — 55% — 11 km/h* ;
   *Mar 2 — nuage-soleil — 18° — 24° — 20% — 62% — 14 km/h* ;
   *Mer 3 — nuage-pluie — 16° — 21° — 80% — 81% — 23 km/h* ;
   *Jeu 4 — nuage-soleil — 17° — 23° — 30% — 66% — 16 km/h* ;
   *Ven 5 — soleil — 18° — 27° — 0% — 52% — 10 km/h*.
   **Les minimales sont teintées en succès, les maximales en avertissement.**

### Carte avant l'arrivée — *Aperçu*

Minuscule, **non cliquable**. Icône nuage-soleil, titre « **Météo** », **une ligne** :
*« 23°C · Ensoleillé »*.

**Ce qui varie :** l'icône et la teinte suivent la donnée réelle — à dessiner pour six
conditions (« Ensoleillé », « Nuageux », « Pluvieux », « Neigeux », « Orageux », « Brumeux ») ;
il existe aussi un libellé « Nuages ». L'unité peut être Celsius ou Fahrenheit. La ville peut
manquer. **L'indice UV n'est pas toujours disponible** — la grille descend alors à deux cases.
Deux descriptions d'état vide sous un même titre (voir *États communs §2*).

---

## Wi-Fi & codes *(wifi-guest)*

> *À quoi sert la surface :* le nom du réseau, le mot de passe révélé selon le calendrier de
> l'hôte, une astuce et les étapes pour se connecter.

**2 surfaces.**

### Carte d'accueil — *Code*

Carte à icône **wifi**, titre « Wi-Fi & codes ». Ouvre le détail en **feuille du bas**.
Contenu identique au détail, **sans le bandeau de sécurité**.

### Détail — *Aperçu* — feuille du bas

1. Un **bandeau d'information** titré « **Réseau invité uniquement** » : « **Utilisez ce Wi-Fi
   pour votre séjour. Ne partagez pas le mot de passe en dehors de votre groupe.** »
2. Le **bandeau du mot de passe non révélé**, le cas échéant : « **Mot de passe pas encore
   disponible** » (voir *États communs §5*).
3. « **Réseau** » + le nom du réseau. Aperçu lu : *Maison-Invites*.
4. « **Mot de passe** » en chasse fixe, puis — **seulement s'il est révélé** — un bouton de
   contour « **Copier le mot de passe** » (toast : « Mot de passe copié »). Aperçu lu :
   *soleil-2026*.
5. L'**astuce** de l'hôte, en petit. Aperçu lu : *« Le réseau 5 GHz est plus rapide dans le
   salon. »*
6. Les **étapes de connexion**, en texte courant. Aperçu lu : *« Choisissez « Maison-Invites »
   dans les réglages Wi-Fi, puis saisissez le mot de passe. »*

> **À noter :** la fiche du module annonce « Réseau, mot de passe et QR code ». **Aucun QR code
> n'est dessiné par le code de cette surface.** Ne rien prévoir tant que ce n'est pas tranché.

---

## Les deux modules sans surface voyageur

**`ical-sync`** et **`nuki`** n'ont **aucune** surface voyageur — ce n'est pas un oubli
d'inventaire. Ni l'un ni l'autre ne possède de code voyageur du tout : leurs seules surfaces
sont côté hôte. `nuki` agit dans le livret, mais **à travers un autre module** : quand l'hôte
le relie à Accès & parking, ce sont les boutons « Déverrouiller » et « Obtenir mon accès » de
la surface **Accès & parking** qui l'appellent. Il n'a pas d'écran à lui.

---

## Ce qui n'a pas pu être établi

Quatre points restent ouverts. Aucun n'empêche de dessiner, mais chacun mérite une réponse
avant la mise au propre.

1. **Le message de l'hôte dans Objets perdus ou trouvés.** La fiche du module le promet, le
   code de la surface ne le produit pas.
2. **Le QR code de Wi-Fi & codes.** La fiche du module l'annonce, le code de la surface ne le
   dessine pas.
3. **La phrase descriptive de Horaires trains.** Ce module est le seul dont la fiche ne dit pas
   à quoi sert sa surface voyageur. Par ailleurs sa gare, ses destinations et ses horaires sont
   figés dans le code : ce sont des données de démonstration, pas encore de la configuration
   d'hôte.
4. **La note de Votre avis** est une liste déroulante d'étoiles, pas une rangée cliquable. À
   confirmer comme intentionnel.

Enfin, un point de méthode : **aucun contenu de ce document n'a été inventé.** Les 17 surfaces
sans aperçu sont décrites par leur structure et leurs libellés fixes, jamais par un exemple de
contenu d'hôte fabriqué. Là où un exemple apparaît, il vient d'un aperçu rendu et il est
annoncé comme tel.
