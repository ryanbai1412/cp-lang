// cf1555e Boring Segments: two pointers over segments sorted by weight; a
// lazy segment tree over the gaps between points counts how often each gap is covered.
let n: 1..=3e5 = read()
let m: 2..=1e6 = read()
domain Gap = 1..m                       // gap g joins points g and g + 1
let segs = read<[n](l: 1..=1e6, r: 1..=1e6, w: 1..=1e6)>().sorted(by: s => s.w)

var cover = SegTree<Gap, Int>(min, fill: 0, act: add)
var lo = 0
var best: ?Int = None
for hi in 0..n:
    cover.apply(Gap(segs[hi].l)..=Gap(segs[hi].r - 1), 1)
    while cover.all > 0:
        best min= segs[hi].w - segs[lo].w
        cover.apply(Gap(segs[lo].l)..=Gap(segs[lo].r - 1), -1)
        lo += 1
print(best!)
