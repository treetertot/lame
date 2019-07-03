# lame

Lame is a multithreaded entity processor for games.

Lame centers on a world type and entity trait.
The world allows adding entities and gives access to a shared field that can be used to store input or to store global state.
The entity type holds the type declarations, construction, and update method.

I'll add decent documentation later, but why would you use this library in the first place?
It's fast. It performed more than 6x the speed of rayon in my very specific 2000 entity (honestly probably edge) test case.
It uses all cores. Using the wonderful num_cpus crate, it makes one worker thread for each hardware thread.
Entities don't have to be send. Many scripting-related types aren't send or sync, which means you need threads to construct and control a local set of entities.
