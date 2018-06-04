# faire example pour les autres

contient
* le storage et safe_maintain pour physic
* la facon particulière de gérer les forces non constantes
* contactor et bodies map parce que c'est une bonne idée

# TODO

* synchronisation turret:
  when shoot look at each cooldown remaining for all turret sync
  then function of distance and cooldown remaining for each is
  like boid force
  (to do it another system just recompute the condition shoot in another system: easy)

  ou alors juste lorsqu'on tire on incite les autre a tirer:
  en diminuant le cooldown max

  ou alors synchronisatino des tempo alors que le rythme n'est pas sur le tempo. par exemple un rythme de 3/4 essai et essai de synchroniser son tempo avec un rythme de 2/4

* faire que le cursor est dessiné par l'os

* pour la doc arreter de mettre des pub (crate) partout juste faire un module doc avec reexport
  je sais pas

# gameplay

* [x] boule de gravité
* [x] monstre tuable ou pas attiré et +- figé lorsque visé (faire toute les possibilité notamment un on l'on ne doit jamais le voir)
* [x] aim/distance dampings velocity
* [x] monstre glacé qui se réveille aléatoirement en fonction de distance: et fonce simplement en direction du héros pour lequel il a été réveillé. si il rencontre un mur alors il s'arrete et attend de voir quand y'en a un en ligne de mire ou va jusqu'a la dernière position ou il l'a vue
* [x] monstre qui avance vers héros et font mal au contact (araigné) sinon bouge random
* [x] chaman: spawn des monstres a une certaine frequence jusqua un certain nombre et se ballade random
* [x] monstre qui fait des cercles pour venir
* [x] faire un trait impl par les composant qui permet de les inséréer ce trait avec une méthode default peut être override pour rajouter des truc (comme le rigid_body par example et uniquement ?) car les force use contructor actuel sont vraiment peut interessant et facilement contournable (une fonction a la création qui randomise). aussi il seront peut être enlevé avec les tempo. si on arrive a le faire tel quel on peut faire du hot reloading facile ? osef
* [x] réécriture tempo
  * [x] circle_to_player faire chgt régulier en tempo
  * [x] faire une partition pour les spawn des uniques spawner
  * [x] idem pour les spawn de chaman ? oui: faire que la loi normale donne le numéro du beat sur lequel il sera spawn
  * [x] faire velocitytoplayer avec update en rythme au lieu de update en continue
  * [x] faire un monstre qui change de direction en rythme et fonce toujours vers le héros. comme un velocity to player mais avec update en rythme (generaliser) au lieu de tout le temps
        il faut pas rester pres de lui sinon on arrive pas a l'esquiver il faut le tuer en passant ou en tirant
* [ ] refaire les unité en meta
  * [ ] bee.rs
  * [ ] chaman.rs
  * [ ] charger.rs
  * [ ] gravity_bomb.rs
  * [x] player.rs
  * [ ] unique_spawner.rs
  * [ ] walker.rs
* [x] faire les armes du héros et enfin faire une vrai première démo
* [x] DebugShapes
* [ ] faire tourelles
      utiliser le remaining time pour mettre a jour l'angle de la tourelle ?
      nombre de beat par tour
      peut être faire aussi des tourelles qui change de sens (même si en fait bof je crois que ça peut être simulé avec un tourelle qui tire irrégulièrement)
* [ ] ¿¿¿¿ faire que les spawner prennent des str au lieu de insertable ????
      on peut aussi enlever le Box<_> dans ce cas
* [ ] faire que le chaman puisse avoir plusieurs insertableobjects
* [ ] peut être faire une dépendance dans les systemes: physic système doit avoir lieu en premier pour pouvoir remplir les contactors et mettre les position au bon endroit pour tous
* [ ] la randomization des activated beat peut se faire dans un composant randomisation qui desactive de facon random !!!!! OUI

## Maybe
* peut être faire que le ToPlayerMemory continue un peu avant de s'arreter. ou pas faut voir si ca casse souvent
  sisi c'est bien, on peut faire juste qu'il continue met un boolean a true et contine deux metre plus loin
  si il ne voit rien avec le boolean a true alors il s'arrete vraiment
  mieux encores: la cible est toujours la position du character+ sa vitesse * 1 ou 2m !!!!!!!!!!!!!!!!!!!!!
* faire sorte de gravity ball mais avec velocity:
  une vitesse angulaire qui va dans la direction du héros
  et ue vitesse linear de norme constante
* faire des boid qui rebondissent contre les murs ou se tue dans un cadre pour simulé une plaine immense

## bof
monstre qui sortent de terre: généraliser uniquespawner avec un composant life optionnel ou pas l'animation peut ne pas trop marcher plutot faire unqieuspawnerrandom uniquespwanerdeterminate ou alors les monstres qui sortent de terre sont simplement des uniquespawnerrandom c'est bien aussi de l'aléatoire -> donc bof

## tourelles
simple bullet (juste sensor avec velocity lorsque creation (+killonproximity?))
tourelle avec rythme
tourelle qui tourne et plus complexe
tourelle continue avec distance et qui tourne et faire un labyrinthe comme dans un jeu précédent

# Fefe

A game that mix hotline miami and left 4 dead.
and shoot em up (esquive de balles lentes)

impl:
* protocols on top of udp TODO
* animations, particles effects ?
* map is layers divide map in grid
  * cell are loaded and saved. (monster are created, layers are drawn)
  * no different grid for different monster and etc...
  * monster can interact with game up to the limit of loaded cells
  OR maybe not necessary

## Mythology Graphisme

use essai inkscape ++ pour les forme même si probablement as avec outils calligraphique quoique.
et faire des applats de couleurs pastel pour les environnements et vifs pour les elements dynamique.

les décors:
* murs écrasé à la Zelda
* endroit innaccessible des objets vue de profil comme dans les estampes chinoise
* beaucoup de choses au sol (qui n'aurait pas vraiment de sens en vrai)

les monstres:
* vue de dessus

monstre: un ver qui avance (boule qui se sépare et avance puis le reste se ramène)

dessin:
* contours noir en mode calligraphie ou pas
* applats de couleurs

animation particle ?
* des éclats noirs
* lorsqu'un monstre meurt sa couleur disparait et les bouts noirs se délient et sont propulsé parfois

le message ? lao tseu ?
gauchiste/anarchisme
thèmes:
* le bonheur
* l'effort
* le travail
* la réussite
* déconstruction d'un ensemble de valeur de droite
* spinoza
* lao tseu

# Music

inspired by Qi meditation music
https://www.youtube.com/watch?v=JXm5-jqkfPY

## Networking

FINALLY: master/client with Option<player> on master
         and client are trusted (shoot is computed on client and server does not check it)

[valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
uses 0.1 "view lag" and snapshot at 0.05 seconds.
latency must be stable ?

## Gameplay

### Monsters

* zombies:
  * when hearing a sound then can run onto the origin of the sound with pathfinding
  * maybe no pathfinding
  * when close to a character they run to the character with pathfinding

maybe use NEAT for all intelligence
TODO: how much does it cost to use a full generated network
      if not that much then all entities will have such a brain
      if quite a lot then only special monster have some

* monstres statues qui s'animent:
  des statues sont dispersé dans une salle parmis des vraies statues aléatoirement
  il se réveille parfois lorsque le héro arrive près

* boules avec gravité vers héros.
  à la manière d'un jeu précédent des boules plus ou moins lentes qui se dirige vers les héros
  lentes: on peut créer des combats au milieu,
  rapides: juste il faut les esquiver

* on peut réutiliser certains monstres de left 4 dead

* boid

* avoider

### Neat

https://github.com/tspooner/rsrl
https://github.com/milanboers/rurel

How do we learn:
* with a basic AI simulating the player

Maybe better:
* pull organism from a site and push the evaluation
* that's very cool as AI is learning from everyone

### Infos to monster

Sound through trigger
Every action create sound that can trigger entities around
(no grid with propagation) just trigger in circle

## New game user story

### Choose Game

* on first start user is assigned a unique ID

* user choose a name that is not necessarily unique

* on start up try to connect to peers servers:
  user is told to which server he is connected and to what he is not
  user can:
  * retry connection for each server
  * add a new server

* user can create a new game with:
  * name
  * password
  * description

* search for games by:
  * members name
  * game name

### In Game

* inner people should invoke new players ?

# idea

enregistrer un mouvement et pouvoir le refaire:
on marche en avant et attaque
on rembobine
on se déplace
on rejoue marche en avant et attaque

on place un point
et on peut se téléporter à la symetrie de ce point

idem s/point/ligne/

transposition avec un monstre

* niveau qui fait changer la caméra lorsqu'on peut pas voir comment
  juste un bruit

* le composant listener peut insérer un composant control lorsqu'il est trigger

* monstres qui se dirige vers héros si en voit et le plus proche

* monstres mine qui faut pas marcher dessus
  ou alors qui sont attiré lorsqu'on est trop proche
  (en fait idem au dessus sauf distance plus courte)

* monstres qui évite d'être dans la ligne de mire

* monstres se réveille brusquement quand proche
* faire la même chose que c'est en fonction de bruit fort

* monstres qui sont attiré lorsqu'on les regarde
* monstres qui sont attiré lorsqu'on les regarde pas
* ou penser l'inverse: ils sont pas attiré quand X mais ils sont figé quand !X

* (un monstre qui saute (ou dash) pour se déplacé)

### monstres finalement:

faire:
* boids:
  évite la ligne de mire
  va vers le héros
  se regroupe
  se regroupe pas trop

* gravité

* gravité sonore

* va vers le héros si en ligne de mir

* monstre attiré quand regarde ou quand regarde pas:
  implémenté a partir de modification induite du regard du héros sur l'entité
  le héros a un pouvoir de paralysie avec son aim sur certain monstres
  le héros a un pouvoir de paralysie avec son non aim sur certain monstres

réflechir:
* tourelles: les facons de tirer est ce que certaines peuvent se déplacé sur les rondes écrite ?
  le lancement des tourelles peuvent correspondre au son au ryhtme régulier :-) aaaah non arrête

* balles: lancé par les tourelles: des mouvements spéciaux
  leurs positions doivent être une fonction paramétré par le temps

* des spawner: pour les glaces qui s'anime: spawner lorsque près du héros. ou plutôt truc spécial
  pour les endroits ou avec des vagues.

* NEAT: faire que les réseaux soit calculé sur GPU et faire les visions du monde

This works for entity that have to go to the player
but what about other entities

also there should be more shootemup like entities
those should be made with turrets launching balls that have special behavior!!!!!!

then

* faire un monstre qui arrive tellement vite quand tu le regarde qu'il faut passer sans le regarder

# arme

arbalete qui tire mais vitesse =/ oo ou alors oo
+ épée

dans une même arme: il faut appuier sur shift pour se mettre en mode arbalete

# musique

* utiliser les boid pour faire de la synchro entre canon
* faire quelque chose en relation avec le fait que chaque note est a une place précise dans la mesure

# note

pour dessiner le cursor faire avec le truc natif ou redessiner un autre:
le truc natif est surement mieux pour le temps de réponse

