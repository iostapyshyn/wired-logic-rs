# [wired-logic-rs](https://iostapyshyn.github.io/wired-logic-rs)
![building](https://github.com/iostapyshyn/wired-logic-rs/workflows/building/badge.svg)

A pixel-based digital circuitry simulator, now powered by Rust and WebAssembly. Inspired by
[wired-logic](https://github.com/martinkirsche/wired-logic).

How does it work?
-----------------
_[Original explanation by martinkirsche](https://github.com/martinkirsche/wired-logic/blob/master/readme.md):_

It scans the image, converts it into a collection of wires, power sources and
transistors and runs a simulation on them as long as the state of the
simulation does not recur. Then it renders the simulation into the animated
gif image.

### The rules

Description | Example
------------|--------
Wires are all pixels of the color from index 1 to 7 within the palette. | ![wire](https://github.com/martinkirsche/wired-logic/raw/master/examples/wire.gif)
A 2x2 pixel square within a wire will make the wire a power source. | ![wire](https://github.com/martinkirsche/wired-logic/raw/master/examples/source.gif)
Wires can cross each other by poking a hole in the middle of their crossing. | ![wire](https://github.com/martinkirsche/wired-logic/raw/master/examples/crossing.gif)
A transistor gets created by drawing an arbitrarily rotated T-shape and, you guessed it, poking a hole in the middle of their crossing. If a transistor's base gets charged it will stop current from flowing. If not, current will flow but gets reduced by one. | ![wire](https://github.com/martinkirsche/wired-logic/raw/master/examples/transistor.gif)

### The idea

> Wired Logic was mainly inspired by Minecraft's Redstone and [Wireworld]. The first prototype even was a cellular automaton like [Wireworld] running as a shader within the GPU where each pixel passed its `charge - 1` on to its neighbours. But it was slow and impractical so I came up with this implementation.

[Wireworld]: https://en.wikipedia.org/wiki/Wireworld

Compilation
-----------
```sh
$ wasm-pack build
$ npm install

$ npm run serve     # to start the webpack dev server
$ npm run bundle    # to create the production bundle
```
