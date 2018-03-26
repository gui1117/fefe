# Fefe

* USE GGEZ ?

* faire recreate swapchain on signal and fence

A game that mix hotline miami and left 4 dead.
and shoot em up (esquive de balles lentes)

impl:
* conrod with own backend
* protocols on top of udp TODO
* vulkano+winit or gfx+winit.
  see how hal evolve and if vulkano could be used on top of hal as it is vulkanic.
  this would be awesome
* nphysics next
* animations, particles effects ?
* map is layers divide map in grid
  * cell are loaded and saved. (monster are created, layers are drawn)
  * no different grid for different monster and etc...
  * monster can interact with game up to the limit of loaded cells
  OR maybe not necessary

## Mythology

use essai inkscape ++ pour les forme même si probablement as avec outils calligraphique quoique.
et faire des applats de couleurs pastel pour les environnements et vifs pour les elements dynamique.

les décors:
* murs écrasé à la Zelda
* endroit innaccessible des objets vue de profil comme dans les estampes chinoise
* beaucoup de choses au sol (qui n'aurait pas vraiment de sens en vrai)

les monstres:
* vue de dessus

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

# Specs

* turret do not create entities in there system: too much mutable storages: better to create them with world directly

* faire un trait pour les entité pouvant être créer depuis une tourelle

* auto insert into/remove from physic world can be made through tracked storage
  insert: physic body need physic world resource to be created.
  remove: after each maintain consistency is done.

  allow one way inconsistency: some entity can be in physic world but not in specs world
  so check with is\_alive

# Components

* body
* life
* playercontrol
* gravitytowardsplayers (Formule avec la distance)
* blade
* sniper
* turret
* vec<animation>: animation can be oriented by body or aim

convention ecrire `life_st` pour `life_storage`

# Graphics

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

# moyen age

arbalete qui tire mais vitesse =/ oo
+ épée

# TODO:

* Implémenter les oreilles qui retiennent position/date
* impl les truc qui se déplace vers toi plus ou moins vite en fonction de la distance
  rigidbody
  ??? DOES THIS CAN BE DONE WITH GRAVITY ????
  YES
* impl les truc qui se déplace vers le son plus ou moins vite en fonction du volume

* then do shapes collision
* then do blind monsters
  va en direction du son
  sur un collision longe le mur du coté le mieux incliné
  si a nouveau obstacle

  ou plutôt faire simplement des longeurs de murs :-)

* conrod: y a-t-il des features a enelver mettre...
* faire rendu de conrod
