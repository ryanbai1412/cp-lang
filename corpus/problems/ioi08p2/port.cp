// Source: IOI 2008 Day 1 (Islands), https://dmoj.ca/problem/ioi08p2
// Problem: n <= 1e6 islands; island i has one bridge of length L_i <= 1e8 to another island, so
//   n bridges. In each connected group take the longest bridge path that visits no island twice;
//   print the sum over the groups (needs 64 bits).
// Solution: each group is one cycle with trees hanging off it. Peel the trees leaf first
//   (Kahn), keeping reach[v], the longest walk down into v's tree, and inside[v], the longest path
//   within it. Then go round each cycle once from a start: for islands a before b the best is
//   reach[a] + reach[b] plus the longer way round, taken from running maxima of reach[a] - at[a]
//   and reach[a] + at[a]. O(n).
// Bounds: bridge.to: 1..=n
let n: Int = read()
domain I = 1..=n
let bridge: [I](to: I, len: Int) = read()

var pending: [I]Int = 0  // unpeeled islands whose bridge leads here
for i in I: pending[bridge[i].to] += 1
var reach: [I]Int = 0   // longest walk from i out into its peeled tree
var inside: [I]Int = 0  // longest path within i's peeled tree
var leaves = [i for i in I if pending[i] == 0]
var next = 0
while next < leaves.len:
    let v = leaves[next]
    next += 1
    let (p, len) = bridge[v]
    inside[p] = max(inside[p], inside[v], reach[p] + reach[v] + len)
    reach[p] max= reach[v] + len
    pending[p] -= 1
    if pending[p] == 0: leaves.push(p)

var total = 0
var seen: [I]Bool = false
for start in I:
    if pending[start] == 0 or seen[start]: continue
    var ring = List<I>()
    var v = start
    while not seen[v]:
        seen[v] = true
        ring.push(v)
        v = bridge[v].to
    let around = sum(bridge[v].len for v in ring)
    // Two islands a, b of the ring, a first: reach[a] + reach[b] plus the
    // longer way round, b's distance from start minus a's, or the rest.
    var best = 0
    var at = 0                                      // distance from start, going round
    var (short, long): (?Int, ?Int) = (None, None)  // best reach[a] - at, reach[a] + at so far
    for b in ring:
        best max= inside[b]
        if short != None: best = max(best, reach[b] + at + short!, reach[b] + around - at + long!)
        short = max(short ?? Int.MIN, reach[b] - at)
        long = max(long ?? Int.MIN, reach[b] + at)
        at += bridge[b].len
    total += best
print(total)
