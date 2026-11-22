// coci21c2p5 Osumnjičeni: how many groups of consecutive suspects, each with pairwise disjoint
// height intervals, cover l..=r. Two pointers give each right end the start of its longest group,
// held as coverage counts on a segment tree. The group starting at s ends just before the first
// suspect whose longest group starts after s; jumping along those links answers a query.
let n = read<1..=200_000>()
domain S = 1..=n
let iv = read<[S](l: 1..=1_000_000_000, r: 1..=1_000_000_000)>()
let (domain C, xs) = compress(chain(iv.map(x => x.l), iv.map(x => x.r)))
fn heights(s: S) -> Range<C> = xs.at(iv[s].l)..=xs.at(iv[s].r)

let held = track(SegTree(max, [C] c => 0, act: add), add: (mut cover, s) => cover.apply(heights(s), 1), remove: (mut cover, s) => cover.apply(heights(s), -1))
let start = iv.keys.maximal(held, ok: h => h.state.all <= 1).map(g => g.first!)
let jumps = Lift([S] s => start.partition_point(b => b <= s).r.map(t => (t, 1)), op: (a, b) => a + b, identity: 0)
for (l, r) in read<[read<1..=200_000>()](S, S)>():
    print(jumps.climb(l, ok: s => s <= r).steps + 1)
