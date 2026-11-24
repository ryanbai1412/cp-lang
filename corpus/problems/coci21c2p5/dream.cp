// COCI 2021/22 C2 P5 Osumnjičeni: suspects stand in order with height ranges. A lineup is
// a contiguous run whose ranges pairwise don't overlap. For each query l..=r, how few lineups
// cover the suspects l..=r?
let n = read<1..=200_000>()
domain S = 1..=n
let height = read<[S](lo: 1..=1_000_000_000, hi: 1..=1_000_000_000)>()
let (domain H, hs) = compress(chain(height.map(h => h.lo), height.map(h => h.hi)))
fn span(s: S) -> Range<H> = hs.index_of(height[s].lo)..=hs.index_of(height[s].hi)

// Two pointers: the longest good run from each s ends just before `after[s]`.
var cover = SegTree(max, [H] _ => 0, act: add)
var after: [S]?S = None
var end = S.cuts.first!
for s in S:
    while let Some(t) = end.r:
        cover.apply(span(t), 1)
        if cover.all > 1:
            cover.apply(span(t), -1)
            break
        end = t.r
    after[s] = end.r
    cover.apply(span(s), -1)

let jump = Lift<S, ()>.of(s => after[s])
let q = read<1..=200_000>()
for (l, r) in read<[q](S, S)>():
    print(1 + jump.climb(l, ok: s => s <= r).steps)
