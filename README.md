<p align="center">
    <img src="header.png" alt="header">
</p>

# <div align="center">Planes of Booty</div>
<p>
    <a href="https://barsoosayque.itch.io/planes-of-booty">
        <img src="https://img.shields.io/badge/download-itch.io-fa5c5c?style=flat-square"
            alt="download from itch.io"></a>
    <img src="https://img.shields.io/badge/release%20date-7%20Jun%202020-red?style=flat-square"
        alt="release date 7 Jun 2020">
</p>

A game made in 21 days for [My little Rougelike #1](https://itch.io/jam/mylittleroguelike1) gamejam.

Table of command line arguments:
|Argument|Description|
|-|-|
|`--debug`|Allow to acces in game debug tools|

*And that's about it.*

# Codegen

So, the game uses special yaml files to generate code to spawn entities and do other cool things with them (like iterating through all the available entities). This is also the same for items and honestly everything you see on the screen besides UI.
Codegen code is kind of messy, you have been warned. But it allows to create new arenas, spawn groups, eneimes, items and particle effects with ease. You can try it, it's so easy even a crab can do it !

# Future

While working on this, we have some ideas that would be cool to implement, but wasn't possible due to time constraints. So, I dumped those ideas here.
And here it is: the List of things worth considering if for some reason development will continue (and also things that were cut):

* Sounds, please
* Obviously, Save & Load
* More on that, dynamic chunk generation and lazy chunk loading
* Open map with island generation
* Actual assets management
* Implement outline using depth buffer and not shaders
* Change `euclid` linear algebra library to [`ultraviolet`](https://github.com/termhn/ultraviolet)
* Implement guns via scripting API (lua, dyon, gluon) or better: yaml codegen somehow ?
* Better codegen using codegen library
* MOAR GUNS AND ENEMIES
