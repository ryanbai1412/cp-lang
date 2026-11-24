// NOI 2017 "Earthworm Queue": worms of length 1..=6 join and split queues; for a string s and k,
// multiply over its k-windows how many times each one appears as k consecutive worms in some
// queue. Count every run of at most 50 consecutive worms by its rolling hash: a join adds the
// runs crossing the new joint, a split removes them.
use std.strings.{Hash61, RollingHash61}

type Z = Mod<998_244_353>
let longest = 50
let base = Some(RollingHash61([0]).base)  // one base for every hash, so they compare

let (n, q) = read<(1..=200_000, 1..=500_000)>()
domain W = 1..=n
let len = read<[W]1..=6>()
enum Op:
    @tag(1) Join(i: W, j: W)
    @tag(2) Split(i: W)
    @tag(3) Ask(s: Str, k: 1..=longest)
var next: [W]?W = None
var prev: [W]?W = None
var seen = Counter<Hash61>()
for w in W: seen.add(RollingHash61([len[w]], base: base).all, 1)

fn crossing(i: W, j: W, sign: Int):
    let left = iterate(Some(i), w => prev[w]).take(longest - 1).collect()
    let right = iterate(Some(j), w => next[w]).take(longest - 1).collect()
    for t in 1..=left.len:
        var h = RollingHash61(left.take(t).rev.map(w => len[w]), base: base)
        for w in right.take(longest - t):
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
            let h = RollingHash61(s.map(c => c.digit), base: base)
            print((0..=s.len - k).map(at => Z(seen[h.get(at..at + k)])).prod())
