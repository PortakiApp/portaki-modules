# Changelog

## 2.3.0

Votre gare et vos destinations.

* Choisissez la gare, les destinations à suivre et le sens des trains.

## 2.2.0

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 2.1.0

Le retard, la fenêtre du séjour, et le bon sens de la flèche.

* Un train en retard le dit : la pastille de fin de rangée porte l'état, et la fiche explique que l'heure affichée tient déjà compte du retard.
* Le tableau est borné au séjour : il ne s'affiche qu'à partir de la veille de l'arrivée.
* La fiche d'un train écrivait le trajet à l'envers au retour, et la carte d'avant l'arrivée montrait les départs au lieu des arrivées.
* Un filtre qui ne laisse aucun train le dit autrement qu'une gare sans train, et la carte d'accueil ne reste plus muette.
* Le tableau dit de quand il date, et un bouton ouvre la billetterie de l'opérateur.

## 2.0.1

La fiche d'un train dit le jour en toutes lettres.

* La tuile « Jour » affichait la date brute de l'API (« 2026-10-04 »). Elle dit « Aujourd'hui » ou « Demain », dans la langue du voyageur.
* La tuile avant l'arrivée a maintenant sa propre adresse : elle ne partage plus celle des autres modules.

## 2.0.0

Les vrais horaires de votre gare.

* Vous donnez le nom de votre gare, et c'est tout : le module la retrouve lui-même.
* Départs et arrivées viennent de la SNCF, en temps réel quand il est disponible.
* Les destinations proposées sortent des trains réels de votre gare : aucune liste à tenir.
* La nuit, le voyageur voit le premier train du matin, annoncé pour demain.
* Le quai et la distance disparaissent : ils étaient écrits en dur, pour un seul logement.

## 1.4.0

Le sens, la gare, la fiche.

* Le sens se choisit d'un doigt : depuis la gare du logement, ou vers elle.
* La gare d'arrivée se cherche en la tapant, au lieu de se trouver dans une barre de pastilles.
* Chaque départ porte sa correspondance en fin de ligne et ouvre sa fiche.

## 1.3.0

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 1.2.0

Les transports dans la langue du voyageur.

* Tous les textes que lit le voyageur existent dans les dix langues du sélecteur du livret.

## 1.0.0 (2026-09-29)

Première version stable. Un tableau des départs de la gare la plus proche, d'un coup d'œil dans le livret.

* Une carte d'accueil montre les prochains départs de la gare la plus proche, toutes destinations confondues.
* Un écran de détail ajoute un en-tête départ/arrivée, des filtres par destination et les prochains départs.
* Rien à régler pour l'hôte : la gare et ses destinations sont livrées avec le module.
* La gare et les horaires sont statiques pour l'instant ; les lire depuis la configuration ou un connecteur viendra ensuite.
