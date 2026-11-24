# Friction report: noip22p3

There is no dream.cp; the ideal is below. The port uses std's `two_edge_components` and `condense` to build the bridge tree. One reverse preorder over `Rooted` blocks counts the held sets per block with `some`/`one`, and the total is scaled by `2 ** m`.

## Dream

```cp
let (domain Block, block) = two_edge_components(land)
let cities = block.counts()            // [Block]Int
let blocks = Rooted(condense(land, block), root: block[City.first!])
...
```

## Counting cities per block is a hand loop
- **Where:** lines 25-26: `var cities: [Block]Int = 0` and `for v in City: cities[block[v]] += 1`.
- **What felt wrong:** "how many cities each block has" is a histogram of a domain-valued array, written as a mutable array and a loop.
- **Goal it hurts:** express.
- **Ideal code:** `let cities = block.counts()` giving `[Block]Int`.
- **General fix:** a std `counts()` on arrays whose element type is a domain, returning an array over that domain. `tally(by:)` returns a map, which is the wrong shape here. Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 236, accepted C++ 1651 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.53 s / 68 MB, accepted C++ 0.84 s / 78 MB, with the same output.
- **Emitted C++:** 29868 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
