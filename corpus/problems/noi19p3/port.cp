// Source: NOI 2019 P3 (Sequence), https://dmoj.ca/problem/noi19p3
// limits: 1000 ms, 511 MB
// Problem: multitest (T <= 10). Given a, b of length n <= 2e5 (sum n <= 1e6, values <= 1e9) and
//   1 <= L <= K <= n, pick K indices c and K indices d sharing at least L indices; print the max
//   of the a-sum over c plus the b-sum over d.
// Solution: a simulated cost flow. Each index is a-only, b-only, both or neither; start with the
//   K largest a+b as both. While more than L are both, demote the worst both index to neither,
//   give a-only and b-only one slot each and re-optimise with two augmentations: try every order
//   of up to three state moves starting from neither, each move taking the best index from an
//   `IndexedHeap` per state pair keyed by what the move loses. The answer is the best total over
//   all rounds; O(n log n) per test.
// original: corpus/submissions/5669058.cpp
// Bounds: n: 1..=2e5; k: 1..=n; l: 1..=k; a, b: 1..=1e9
use std.heap.{IndexedHeap}

enum Side: A | B | Both | Neither

@multitest
let n: 1..=2e5 = read()
let k: 1..=n = read()
let l: 1..=k = read()
domain I = 0..n
let a: [I]1..=1e9 = read()
let b: [I]1..=1e9 = read()

fn worth(s: Side, i: I) -> Int:
    match s:
        A => a[i]
        B => b[i]
        Both => a[i] + b[i]
        Neither => 0

var moves = [Side] _ => [Side] _ => IndexedHeap<I, Int>()  // [s][t]: indices in s by what s -> t loses
var side: [I]Side = Neither
var count: [Side]Int = 0
var room: [Side]Int = 0
var total = 0

fn enter(i: I, s: Side):
    side[i] = s
    count[s] += 1
    for t in Side:
        if t != s and (t != Neither or s == Both): moves[s][t].set(i, worth(s, i) - worth(t, i))

fn shift(i: I, to: Side):
    let from = side[i]
    for t in Side: moves[from][t].remove(i)
    count[from] -= 1
    room[from] += 1
    room[to] -= 1
    total += worth(to, i) - worth(from, i)
    enter(i, to)

fn route(order: List<Side>) -> ?(gain: Int, path: List<Side>):
    var (from, gain, path) = (Neither, 0, List<Side>())
    for to in order:
        if let (_, loss) = moves[from][to].peek:
            gain -= loss
            path.push(to)
            if room[to] > 0: return (gain:, path:)
            from = to
        else:
            return None
    None

fn augment():
    var best: ?(gain: Int, path: List<Side>) = None
    for order in [A, B, Both].permutations():
        if let r = route(order):
            if best == None or r.gain > best!.gain: best = r
    if let r = best:
        var from = Neither
        for to in r.path:
            shift(moves[from][to].peek!.0, to)
            from = to

for i in I: enter(i, Neither)
room[Both] = k
repeat k: shift(moves[Neither][Both].peek!.0, Both)
var most = total
while count[Both] > l and count[Neither] > 0:
    shift(moves[Both][Neither].peek!.0, Neither)
    room[Both] -= 1
    room[A] += 1
    room[B] += 1
    augment()
    augment()
    most max= total
print(most)
