# Friction report: ucrpc21d

There is no dream.cp; the ideal is below. The port reads the canvas a line at a time and unites each room with its right and lower neighbour, in a `Dsu<Room>` over the product domain `(Row, Col)`, when the character between them is open. The max output differs from the C++ only by the C++'s trailing space.

## Dream

```cp
let canvas: [0..=2 * r, 0..=2 * c]Char = read()
for room in Room:
    for (next, wall) in [(room.right, canvas.between(room, room.right)), ...]: ...
```

## Wall characters are found by canvas coordinate arithmetic
- **Where:** lines 20-26: `Row(y / 2)`, `line[2 * Int(k)]` and `line[2 * Int(j) + 1]`, with `y % 2` deciding which kind of line it is.
- **What felt wrong:** the canvas is a 2x-scaled picture of the rooms. Each wall is reached by doubling a room's ordinal and adding an offset, which is easy to get off by one.
- **Goal it hurts:** hard-to-get-wrong.
- **Ideal code:** a scaled view of the canvas indexed by rooms: `canvas.at(room, side: Right)`.
- **General fix:** none in std yet. A "cells and walls" grid view would serve this and other drawn-maze inputs. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 211, accepted C++ 686 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.03 s / 6 MB, accepted C++ 0.07 s / 6 MB.
- **Emitted C++:** 23127 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
