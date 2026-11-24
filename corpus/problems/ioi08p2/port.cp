// Source: IOI 2008 Day 1 (Islands), https://dmoj.ca/problem/ioi08p2
// Problem: n <= 1e6 islands; island i has one bridge of length L_i <= 1e8 to another island, so
//   n bridges. In each connected group take the longest bridge path that visits no island twice;
//   print the sum over the groups (needs 64 bits).
// Solution: each group is one cycle with trees hanging off it (a `Functional` graph). Fold the
//   trees deepest island first, keeping reach[v], the longest walk down into v's tree, and
//   inside[v], the longest path within it. Then go round each cycle once from a start: for islands a before b the best is
//   reach[a] + reach[b] plus the longer way round, taken from running maxima of reach[a] - at[a]
//   and reach[a] + at[a]. O(n log n) for the depth order.
// limits: 1500 ms, 1024 MB
// Bounds: n: 2..=1e6; bridge.to: 1..=n; bridge.len: 1..=1e8
let n: 2..=1e6 = read()
domain I = 1..=n
let bridge: [I](to: I, len: 1..=1e8) = read()

let islands = Functional([I] i => bridge[i].to)

var reach: [I]Int = 0   // longest walk from i out into its tree
var inside: [I]Int = 0  // longest path within i's tree
for v in sorted([v for v in I if not islands.on_cycle(v)], by: v => desc(islands.depth(v))):
    let (p, len) = bridge[v]
    inside[p] = max(inside[p], inside[v], reach[p] + reach[v] + len)
    reach[p] max= reach[v] + len

var total = 0
for ring in islands.cycles():
    let around = ring.sum(v => bridge[v].len)
    // Two islands a, b of the ring, a first: reach[a] + reach[b] plus the
    // longer way round, b's distance from the start minus a's, or the rest.
    var best = 0
    var at = 0                                      // distance from the start, going round
    var (minus, plus): (?Int, ?Int) = (None, None)  // best reach[a] - at, reach[a] + at so far
    for b in ring:
        best max= inside[b]
        if let s = minus and let l = plus: best = max(best, reach[b] + at + s, reach[b] + around - at + l)
        minus max= reach[b] - at
        plus max= reach[b] + at
        at += bridge[b].len
    total += best
print(total)
