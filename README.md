# Fefe

a game that mix hotline miami and left 4 dead.

## Networking

The game is peer to peer.

Each member of the session control its character and part of the monsters.

Too be smouth remote entities must be interpolated between snapshots.
We should make the delta past time configurable and number of snapshot per second configurable too.

Event send to peers are snapshot of controlled entities (property updates in UE4) and remote procedure calls.
Remote procedure calls are typically damage to remote entity.

Example:
* I shoot on one of my entities: it is killed instantly and others will have the information delta time after.
* I shoot on an entity of another one: shoot and animation are played directly and information is send.
  if an attack is blocked or entity is dead then it should take effect instantly because otherwise we could be damaged
  by the entity while having killed or blocked it.
  Ok maybe we don't fix it and there is delay between shoot sound/blood effect And monster dead animation.

## Gameplay

### Sound

every action create sound that can trigger entities around

### Monsters

* zombies:
  * when hearing a sound then can run onto the origin of the sound with pathfinding
  * when close to a character they run to the character with pathfinding
