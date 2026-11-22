// NOI 2017 "Earthworm Queue": worms of length 1..=6 join and split queues;
// for a string s and k, multiply over its k-windows how many times each one
// appears as k consecutive worms in some queue.
type Z = Mod<998244353>
enum Op = Join(i: W, j: W) | Split(i: W) | Ask(s: Str, k: 1..=50)

let (n, q) = read<(1..=2e5, 1..=5e5)>()
domain W = 1..=n
let len = read<[W]1..=6>()
var next: [W]?W = None
var prev: [W]?W = None
var seen = Counter<PolyHash>(W.map(w => PolyHash.of([len[w]])))

fn crossing(i: W, j: W, sign: Int):
    let left = iterate(Some(i), w => prev[w]).take(49).collect()
    let right = iterate(Some(j), w => next[w]).take(49).collect()
    for (a, tail) in left.prefixes().skip(1).items:
        var h = PolyHash.of(tail.rev.map(w => len[w]))
        for w in right.take(50 - tail.len):
            h = h.push(len[w])
            seen.add(h, sign)

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
            print(s.windows(k).product(of: w => Z(seen[PolyHash.of(w.map(c => c.digit()))])))
