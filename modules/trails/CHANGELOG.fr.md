# Changelog

## 0.7.0

Aller simple et bornes des mesures.

* Un itinéraire peut être un aller simple.
* Jusqu'à 30 itinéraires ; durée, distance et dénivelé hors bornes sont signalés sous le champ.
* Un niveau manquant bloque la publication ; un départ sans position déclenche un avertissement.

## 0.6.0

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 0.5.3

Pas de filtre quand il n'y a qu'un niveau.

* La carte d'accueil posait une pastille de niveau même quand tous les itinéraires partagent le même : elle répétait ce que chaque rangée dit déjà.
* Le niveau reste en fin de rangée, là où il se lit.

## 0.5.2

Correction de la vitrine.

* L'exemple montré dans la vitrine du module nomme et situe correctement le logement ; rien ne change pour l'hôte ni pour le voyageur.

## 0.5.1

Montée technique.

* Le module est passé à la dernière base technique de Portaki ; rien ne change pour l'hôte ni pour le voyageur.

## 0.5.0

Les mesures viennent de la trace.

* « Lire les mesures de la trace » remplit la distance, le dénivelé et le type depuis le GPX déposé.
* Les valeurs restent modifiables : la trace propose, vous décidez.
* Une trace illisible ne change rien — vos mesures restent celles que vous avez saisies.
* Au-delà de huit itinéraires, la liste gagne un filtre par niveau.

## 0.4.0

Le tracé, et la photo.

* Le tracé du GPX que vous déposez se dessine sur le plan de la fiche.
* Le voyageur télécharge la trace pour l'ouvrir dans son application de randonnée.
* Chaque itinéraire peut porter une photo, en tête de sa fiche.

## 0.3.0

Trois mesures, trois repères.

* La distance porte une boussole et le dénivelé une flèche : les trois mesures se distinguent d'un coup d'œil.

## 0.1.0

Première version. Les randonnées que vous conseillez, classées par niveau — et les trois mesures
qui font qu'un voyageur en choisit une plutôt qu'une autre.

* Trois niveaux seulement, les mêmes partout : facile, moyen, sportif. Affichés en forme et en texte (▲, ▲▲, ▲▲▲), jamais en couleur — une difficulté n'est pas une alerte.
* La fiche donne la durée, la distance, le dénivelé et le type, le plan du départ, et le bouton qui ouvre la trace chez Visorando ou l'IGN.
* Quand le départ n'est pas devant le logement, un second bouton y emmène le voyageur.
* Les départs posent leurs repères sur la Carte du livret.
