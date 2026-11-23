// NOI 2017 "Earthworm Queue": worms of length 1..=6 join and split queues;
// for a string s and k, multiply over its k-windows how many times each one
// appears as k consecutive worms in some queue. Windows are at most 50 long,
// so a join or split changes only the windows crossing the cut: count them in
// a hash counter, adding or removing each crossing window.
use std.strings.{HashVal}

type Z = Mod<998244353>
let longest = 50
enum Op = @tag(1) Join(i: W, j: W) | @tag(2) Split(i: W) | @tag(3) Ask(s: Str, k: 1..=longest)

let (n, q) = read<(1..=2e5, 1..=5e5)>()
domain W = 1..=n
let len = read<[W]1..=6>()
var next: [W]?W = None
var prev: [W]?W = None
let hasher = RollingHash.new(List.filled(longest, 0))
var seen = Counter<HashVal, Int>(0)
for w in W:
    seen.add(hasher.hash_of([len[w]]), 1)

/// Count (`sign = 1`) or forget (`sign = -1`) every window with `j` right behind `i`.
fn crossing(i: W, j: W, sign: Int):
    let left = List(iterate(Some(i), w => prev[w]).take(longest - 1))
    let right = RollingHash.new(List(iterate(Some(j), w => next[w]).take(longest - 1)).map(w => len[w]), bases: hasher.bases)
    var tail = HashVal(0, 0)
    for (a, w) in left.items:
        tail = hasher.combine(hasher.hash_of([len[w]]), tail, a)
        for b in 1..=min(right.len, longest - 1 - a):
            seen.add(hasher.combine(tail, right.get(0..b), b), sign)

for op in read<[q]Op>():
    match op:
        Join(i, j) =>
            crossing(i, j, 1)
            (next[i], prev[j]) = (Some(j), Some(i))
        Split(i) =>
            let j = next[i]!
            crossing(i, j, -1)
            (next[i], prev[j]) = (None, None)
        Ask(s, k) =>
            let h = RollingHash.new(s.map(c => c.digit()), bases: hasher.bases)
            print((0..=s.len - k).map(at => Z(seen.get(h.get(at..at + k)))).prod())
