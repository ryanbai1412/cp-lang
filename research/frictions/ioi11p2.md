# Friction report: ioi11p2

There is no dream.cp; the ideal is below. The port is the editorial's small-to-large merge over `tree.at.rev`, with no recursion. `reach[v]` maps a distance from the root to the fewest roads, and `fewest` is a `?Int` printed as `?? -1`.

## Dream

```cp
for v in tree.at.rev:
    var here: HashMap<Int, Int> = [tree.dist(tree.root, v): tree.depth(v)]
    for child in tree.children(v):
        var there = take(mut reach[child])
        if there.len > here.len: swap(mut here, mut there)
        for (far, hops) in there:
            if let other = here.get(k + 2 * far_v - far): fewest min= hops + other - 2 * deep_v
        for (far, hops) in there: here[far] min= hops
    reach[v] = here
```

## Moving a child's map out takes a swap with an empty one
- **Where:** lines 418-419 and 425: `var there: HashMap<Int, Int> = []` then `swap(mut there, mut reach[child])`, and `swap(mut reach[v], mut here)`.
- **What felt wrong:** the intent is "take the child's map, leaving it empty" and "store mine". Spelled as swaps with an empty map, the move costs a declaration and reads as a two-way exchange.
- **Goal it hurts:** express.
- **Ideal code:** `var there = take(mut reach[child])` and `reach[v] = here` (a move of a local at its last use).
- **General fix:** a std `take(mut x)` that returns the value and leaves the type's empty value; std has no `take` today. It helps every small-to-large merge (cf DSU on tree ports). Owner: stdlib.
- **Severity:** low.

## Measurements

- **Tokens:** port 337, accepted C++ 666 (comments excluded).
- **Runtime and memory on gen.py --max:** port 0.38 s / 25 MB, accepted C++ 0.35 s / 20 MB, with the same output.
- **Emitted C++:** 30924 bytes.
- **Agreement:** `cpl stress` against the accepted C++ agrees on 30 gen.py cases at size 30.
