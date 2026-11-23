// coci21c2p5 Osumnjičeni: how many groups of consecutive suspects, each with pairwise disjoint
// height intervals, cover l..=r. Two pointers over a max-coverage segment tree give the longest
// good group from each suspect; a query counts greedy jumps from one group to the next start.
let n = read<1..=200_000>()
domain S = 1..=n
let iv = read<[S](l: 1..=1_000_000_000, r: 1..=1_000_000_000)>()
let (domain C, xs) = compress(chain(iv.map(x => x.l), iv.map(x => x.r)))
fn heights(s: S) -> Range<C> = xs.at(iv[s].l)..=xs.at(iv[s].r)

var cover = SegTree(max, [C] c => 0, act: add)
var after: [S]?S = None // the first suspect past the longest good group starting at s
var end = S.cuts.first!
for s in S:
    while let Some(t) = end.r:
        cover.apply(heights(t), 1)
        if cover.all > 1:
            cover.apply(heights(t), -1)
            break
        end = t.r
    after[s] = end.r
    cover.apply(heights(s), -1)

let jumps = Lift<S, ()>.of(s => after[s])
let q = read<1..=200_000>()
for (l, r) in read<[q](S, S)>():
    print(jumps.climb(l, ok: s => s <= r).steps + 1)
