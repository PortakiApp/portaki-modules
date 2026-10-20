# Changelog

## 2.12.0

Le catalogue et les demandes passent sur la nouvelle base de Portaki.

* Le catalogue et les réglages des demandes sont repris à la première ouverture ; les demandes déjà reçues restent liées à leurs produits.
* « Répondre » montre la dernière réponse au-dessus du champ ; « Prévu » et « Marquer livré » changent le statut de la demande.
* Décocher une demande dans la frise la rouvre, et une même réponse envoyée deux fois ne part qu'une fois.
* Un produit sans nom, que le voyageur ne voyait pas, n'est pas repris.
* Le bouton d'import CSV, annoncé « Bientôt », est retiré.

## 2.11.8

Les messages du formulaire dans toutes les langues.

* Les erreurs sous les champs et les points à compléter avant publication s'affichent dans la langue de l'hôte, plus seulement en français ou en anglais.

## 2.11.1

Montée technique.

* Le module passe à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 2.11.0

Les produits en tableau, et les demandes au même endroit.

* Les produits se gèrent dans un tableau : recherche, filtres « Demandés récemment » et « Jamais demandés », chaque ligne s'ouvre dans un panneau.
* Chaque produit dit où le trouver, et le voyageur le lit avant de demander ; l'emoji est proposé d'après le nom, avec une liste de départ en un clic.
* Vous choisissez d'être prévenu par e-mail ou non ; sans délai écrit, le voyageur lit « dans la journée ».
* Les demandes reçues s'affichent en table, « Répondre » écrit au voyageur par e-mail, et celles à traiter apparaissent enfin dans À venir.
* Les identifiants et codes du module sont désormais tirés au hasard par la plateforme : ils ne peuvent plus se répéter ni se deviner.

## 2.10.0

Les produits de base dans les équipements.

* Dès qu'un produit nommé est à votre liste, les produits de base s'ajoutent aux équipements de la page du logement, sans le détail des produits.
* Le catalogue de produits s'enregistre de nouveau : l'ajout des emojis le faisait échouer.
* Nom ou délai de plus de 40 caractères, plus de 30 produits, demandes par séjour hors de 1 à 20 : l'erreur s'affiche avant publication.
* Une demande passe « Prévu » puis « Livré », avec votre réponse au voyageur ; les demandes à traiter s'affichent dans À venir.
* Un catalogue vide ne bloque plus la publication : le livret affiche simplement qu'il n'y a rien à demander.

## 2.9.0

Encadrez les demandes de réassort.

* Vous pouvez fermer les demandes : la carte informe alors seulement le voyageur.
* Le nombre de demandes par séjour se règle de 1 à 20 (5 par défaut) ; au-delà, le voyageur est invité à vous contacter.
* Le catalogue accepte jusqu'à 30 produits.

## 2.8.0

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 2.7.0

Un produit déjà signalé le dit.

* Une tuile dont le produit a déjà été signalé porte « Signalé », avec le niveau annoncé : on ne redemande pas deux fois la même chose.
* Le voyageur voit ce que l'hôte a déjà reçu, et l'hôte ne reçoit plus le même signalement en double.

## 2.6.0

Un en-tête sur la feuille.

* La feuille s'ouvre sur « Il manque quelque chose ? » et dit ce que l'hôte en fera, au lieu du seul mot « Consommables ».

## 2.5.0

Un emoji par produit, et votre délai annoncé.

* Chaque produit porte son emoji : la grille du voyageur se lit d'un coup d'œil au lieu de huit colis identiques.
* La liste de départ arrive déjà illustrée.
* Nouveau réglage : le délai de réapprovisionnement que vous annoncez. Le voyageur le lit avant d'envoyer son signalement.
* Sans délai renseigné, il lit comme avant que son message part bien.

## 2.4.1

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 2.4.0

Un formulaire plus clair.

* « Plusieurs choix possibles » est écrit : un voyageur à qui il manquait deux produits envoyait deux signalements.
* « Il n'y en a plus » porte un signe d'alerte : les deux niveaux ne demandent pas la même chose.
* Une ligne de clôture dit que vous êtes averti dès l'envoi.

## 2.3.0

La grille de l'hôte dépasse ses huit cases.

* L'hôte peut saisir plus de huit articles dans la grille, jusqu'à vingt-quatre.

## 2.2.0

La grille se coche plus d'une fois.

* Le voyageur peut signaler plusieurs articles manquants dans la même grille, sans la recharger.
* Tous les textes que lit le voyageur existent dans les dix langues du sélecteur du livret.

## 1.0.0 (2026-09-29)

Première version stable. Le voyageur signale ce qui manque, pour que l'hôte réapprovisionne avant l'arrivée suivante plutôt qu'après la plainte.

* Le voyageur signale qu'un consommable manque ou sera bientôt vide — papier toilette, café, savon — avec une précision facultative.
* Chaque signalement envoie un e-mail à l'hôte et apparaît dans le détail du séjour, à marquer réapprovisionné après passage.
* L'hôte gère la liste des consommables depuis un onglet Consommables, pré-rempli des articles courants.
