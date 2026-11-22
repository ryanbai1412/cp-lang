// coci21c2p5 Osumnjičeni: how many groups of consecutive suspects, each with pairwise disjoint
// height intervals, cover l..=r. Two pointers keyed by left end give the longest good group
// from each suspect; greedy lineups jump from one group's end to the next start.
let n = read<1..=200_000>()
domain S = 1..=n
let iv = read<[S](l: 1..=1_000_000_000, r: 1..=1_000_000_000)>()
let (domain C, xs) = compress(chain(iv.map(x => x.l), iv.map(x => x.r)))
fn heights(s: S) -> Range<C> = xs.at(iv[s].l)..=xs.at(iv[s].r)
let held = track(SegTree(max, [C] c => 0, act: add),
    add: (mut cover, s) => cover.apply(heights(s), 1),
    remove: (mut cover, s) => cover.apply(heights(s), -1))
let group = iv.keys.maximal(held, ok: h => h.state.all <= 1, anchor: Anchor.Start)
let jumps = Lift.of(s => group[s].last!.next)
for (l, r) in read<[read<1..=200_000>()](S, S)>():
    print(jumps.climb(l, ok: s => s <= r).steps + 1)
