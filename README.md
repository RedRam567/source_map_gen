# source_map_gen

[![Crates.io](https://img.shields.io/crates/v/source_map_gen)](https://crates.io/crates/source_map_gen)
[![docs.rs](https://img.shields.io/docsrs/source_map_gen)](https://docs.rs/source_map_gen/0.1.0/source_map_gen)

A WIP map generator for Source Engine games.

# HIGHLY WIP

Pretty much everything is subject to change.
Some paths in test modules are hard coded to my fs and assume l4d2 and tf2 are installed.
Currently no procedural generation.
I hope it get it generic enough to work in any source game or even other games.
And have it generate the whole map and all the decorations and maybe even
navmesh.

# Current Features:

- vmf parsing
- vmf writing
- cubes, cones, cylinders, spheres
- basic entities
- example pallet map bin crate

# Generated Examples

Complex cyliner:
![Complex cyliner](https://cdn.discordapp.com/attachments/836787072768671786/1106754223976239125/image.png)
Ellipsoid:
![Ellipsoid](https://cdn.discordapp.com/attachments/836787072768671786/1121226118482116688/image.png)
Clamshell cylinder:
![Clamshell cylinder](https://cdn.discordapp.com/attachments/836787072768671786/1106978296366891048/image.png)
Pallet Map:
![Pallet Map](https://cdn.discordapp.com/attachments/836787072768671786/1094669378219425883/image.png)
