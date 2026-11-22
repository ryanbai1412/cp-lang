// NOI 2017 "Earthworm Queue": worms of length 1..=6 join and split queues;
// for a string s and k, multiply over its k-windows how many times each one
// appears as k consecutive worms in some queue. Windows are at most 50 long,
// so a join or split changes only the windows crossing the cut: count them in
// a hash counter, adding or removing each crossing window.
use std.strings.{HashVal}

type Z = Mod<998244353>
enum Op = Join(i: W, j: W) | Split(i: W) | Ask(s: Str, k: 1..=50)

let (n, q) = read<(1..=2e5, 1..=5e5)>()
domain W = 1..=n
let len = read<[W]1..=6>()
var next: [W]?W = None
var prev: [W]?W = None
let hasher = RollingHash.new(List<Int>())
var seen = Counter<HashVal, Int>(0)
for w in W:
    seen.add(hasher.hash_of([len[w]]), 1)

fn crossing(i: W, j: W, sign: Int):
    let left = List(iterate(Some(i), w => prev[w]).take(49))
    let right = List(iterate(Some(j), w => next[w]).take(49))
    for a in 1..=left.len:
        var h = RollingHash.new(left.take(a).rev.map(w => len[w]))
        for w in right.take(50 - a):
            h.push(len[w])
            seen.add(h.all, sign)

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
            print(s.windows(k).map(w => Z(seen.get(hasher.hash_of(w.map(c => c.digit()))))).prod())
