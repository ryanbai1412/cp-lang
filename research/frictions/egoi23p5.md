# Friction report: egoi23p5

Rethought. The v0.2 port found the insertion point with a reversed loop that kept overwriting `at` (a `path.len` default as a sentinel) and rebuilt the whole line into a new list on every insert. The v0.3 port reads each ranking as a typed `[me.ord]G` array, marks the liked first half from a slice, picks the place in one `if`/`elif`/`else` expression with `find` for the first liked adjacent pair, and calls `line.insert`.

No language gap.

## Measurements

gen.py --max (n = 1000): port 0.00 s, accepted C++ 0.01 s, same output.
