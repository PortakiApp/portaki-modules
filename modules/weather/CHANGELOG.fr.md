# Changelog

## 1.6.0

La météo sur la page publique du logement.

* Un bloc Climat peut s'afficher sur la page publique du logement : la température du jour, sans rien du séjour.
* Les températures moyennes des six mois de la saison peuvent s'y ajouter, quand OpenWeather les fournit.
* L'hôte l'active dans une nouvelle carte « Page publique » des réglages Météo ; il est masqué par défaut.

## 1.5.0

Les réglages de la météo, revus pour l'hôte.

* L'hôte choisit le nom affiché sous les prévisions, par exemple « Chamrousse 1750 » plutôt que la commune.
* Une autre position que l'adresse du logement peut être choisie quand elle est imprécise (montagne, île).
* La carte météo avant l'arrivée peut être masquée.
* Les températures suivent la langue du voyageur : degrés Fahrenheit en anglais américain, Celsius ailleurs.
* Une erreur de saisie s'affiche sous le champ et empêche de publier ; une position lointaine est signalée.

## 1.4.3

La météo du séjour, pas cinq jours en dur.

* Les prévisions couvrent le séjour, de la veille de l'arrivée au lendemain du départ : une semaine à la mer s'arrêtait le mercredi.
* Aucune journée passée n'est montrée pendant le séjour.

## 1.4.2

Correctif interne.

* La tuile avant l'arrivée a maintenant sa propre adresse : elle ne partage plus celle des autres modules.

## 1.4.1

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 1.4.0

Jour par jour.

* Les prévisions se lisent en lignes : l'icône, le jour, le temps, les deux températures en fin de ligne.
* Vent, pluie et humidité passent en sous-titre, avec leur mot — la grille à sept colonnes les réduisait à trois caractères.
* La bande de la carte d'accueil met chaque jour en boîte, aujourd'hui en couleur.

## 1.3.0

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 1.2.0

L'e-mail d'arrivée respecte l'unité choisie.

* L'e-mail du jour d'arrivée donne la température dans l'unité choisie par l'hôte.
* Tous les textes que lit le voyageur existent dans les dix langues du sélecteur du livret.

## 1.0.0 (2026-09-29)

Première version stable. La météo de l'endroit où le voyageur se trouve vraiment, dans le livret et dans l'e-mail d'arrivée.

* Le voyageur voit la température et le temps du moment, l'indice UV quand il est disponible, et les prévisions à 5 jours.
* Un résumé de la météo figure aussi dans l'e-mail du jour d'arrivée.
* L'hôte choisit seulement les unités et la fréquence de mise à jour ; les prévisions suivent l'adresse du logement.
* Les données météo viennent de Portaki ; l'hôte n'a rien à connecter.
