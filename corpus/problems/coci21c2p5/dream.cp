// COCI 2021/22 C2 P5 Osumnjičeni: suspects stand in order with height ranges. A lineup is
// a contiguous run whose ranges pairwise don't overlap. For each query l..=r, how few lineups
// cover the suspects l..=r?
domain S = [read Int]
let height = read [S](lo: Int, hi: Int)
let (domain H, _) = compress(height.flat_map(h => [h.lo, h.hi]))
let span = height.map(h => H(h.lo)..=H(h.hi))

let cover = track(SegTree(max, [H]0, act: add),
    add: (mut t, s) => t.apply(span[s], 1),
    remove: (mut t, s) => t.apply(span[s], -1))
let lineup = S.longest_from(cover, ok: t => t.all <= 1)   // longest good run starting at each s
let jump = Lift.of(s => lineup[s].end.r)

for (l, r) in read [read Int](S, S):
    print(1 + jump.climb(l, ok: s => s <= r))
