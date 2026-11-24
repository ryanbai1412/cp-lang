# Friction report: nccc8s4

## Dream

```cp
let atoms: [Atom, Side]Label = read()
let next = Graph<Label>.relation((x, y) => Atom.any(a => joins(a, x, y)))
print(Int(find_cycle(next) != None))
```

## Coloured DFS becomes `find_cycle`

The v0.2 port ran a recursive three-colour DFS with an `Int` state per vertex and a mutable
`has_cycle` flag. The rewrite builds the label graph as `[Label]List<Label>` and asks std
`find_cycle`.

## Labels as a signed domain

`domain Label = -26..=26` types the read. The `+ 26` / `26 -` offsets are gone. Negating a
label (`Label(-Int(x))`) needs a round trip through `Int`, because `-` is not defined on a
domain element even when the domain is symmetric.

## Dense relation, then lists

The joins are collected in a `[Label, Label]Bool` to drop duplicates and then turned into
adjacency lists. A `Graph` built straight from a dense boolean relation would skip that step.

## No accepted C++

The only AC submission is Java, so `orig.cpp` is a hand-written C++ reference (the same DFS).
30 random cases agree with the v0.2 port and 30 with `orig.cpp` (new `gen.py`). Corpus tests pass.
