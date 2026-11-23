// cf1555e Boring Segments: two pointers over segments sorted by weight; a
// lazy segment tree over the gaps between points counts how often each gap is covered.
let n: 1..=3e5 = read()
let m: 2..=1e6 = read()
domain Gap = 1..m                       // gap g joins points g and g + 1
let segs = read<[n](l: 1..=1e6, r: 1..=1e6, w: 1..=1e6)>().sorted(by: s => s.w)

let covered = track(
    SegTree<Gap, Int>(min, fill: 0, act: add),
    (mut t, s) => t.apply(Gap(s.l)..=Gap(s.r - 1), 1),
    (mut t, s) => t.apply(Gap(s.l)..=Gap(s.r - 1), -1),
)
let shortest = segs.minimal(state: covered, ok: t => t.state.all > 0)

var best: ?Int = None
for piece in shortest:
    if let p = piece: best min= segs[p.last!].w - segs[p.first!].w
print(best!)
