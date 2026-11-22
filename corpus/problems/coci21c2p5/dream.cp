// coci21c2p5 Osumnjičeni: queries ask how many groups of consecutive suspects, each group with
// pairwise disjoint height intervals, cover l..=r. Two pointers give each start the end of its
// longest group; jumping greedily along those ends with binary lifting answers a query.
let n = read<1..=200_000>()
let iv = read<[n](l: 1..=1_000_000_000, r: 1..=1_000_000_000)>()
let (domain C, xs) = compress(chain(iv.map(x => x.l), iv.map(x => x.r)))
var cover = SegTree(max, [C] c => 0, act: add)
var after: [0..=n]?(Int, Int) = none
var j = 0
for i in 0..n:
    while j < n and cover.fold(xs.at(iv[j].l)..=xs.at(iv[j].r)) == 0:
        cover.apply(xs.at(iv[j].l)..=xs.at(iv[j].r), 1)
        j += 1
    after[i] = (j, 1)
    cover.apply(xs.at(iv[i].l)..=xs.at(iv[i].r), -1)
let jumps = Lift(after, op: (a, b) => a + b, identity: 0)
for (l, r) in read<[read<1..=200_000>()](1..=n, 1..=n)>():
    print(jumps.climb(l - 1, ok: c => c <= r - 1).steps + 1)
