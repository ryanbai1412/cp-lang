// n: 4096
// let: n = \log n; k = 2
use std.graph.{chromatic_number, de_bruijn}
use std.bitset.{Bitset}
// Exponential algorithms on log2(n) vertices: an odd cycle with a chord, so colouring needs 3.
let n: Int = read()
var lg = 0
while 1 << (lg + 1) <= n: lg += 1
domain V = 0..lg
let adj = [V] v => Bitset.of([V((v.ord + 1) % lg), V((v.ord + lg - 1) % lg)])
var s = 0
for _ in 0..32:
    s += chromatic_number(adj) // timed: graph.chromatic_number
for _ in 0..640:
    s += de_bruijn(2, lg).len // timed: graph.de_bruijn
print(s)
