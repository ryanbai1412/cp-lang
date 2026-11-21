// Codeforces 2157 G: Isaac's Queries (interactive, dream).
// `? u v` is msb(P[u-1] ^ P[v]) over the n+1 prefix xors: the depth at which
// two cuts part in the binary trie of the P's. Split the cuts top-down. In a
// group, ask each cut against the farther of the group's two ends, so every
// query spans at least half the group; the largest answer is the group's top
// bit, an answer below it keeps a cut on that end's side, and every pair across
// the split answers the top bit. Random a keeps groups wide: about 8 coins a test.
@!interactive
@!multitest
type Msb = -1..30                       // a -2 from the judge fails this read and ends the run

let n: 3..=100 = read()
domain Cut = 0..=n
let msb = memo((x: Cut, y: Cut) => ask<Msb>("?", min(x, y) + 1, max(x, y)))
var bit: SymGrid<Cut, Msb> = [:]

fn split(group: List<Cut>):
    if group.len < 2: return
    let (lo, hi) = (group.first, group.last)
    let toward = x => if x - lo >= hi - x: lo else: hi
    let top = group.max(x => msb(x, toward(x)))
    if top == -1:                       // one trie leaf: every prefix xor is equal
        for (x, y) in group.choose(2): bit[x, y] = -1
        return
    let hi_across = msb(lo, hi) == top
    let across = x => (toward(x) == hi and hi_across) != (msb(x, toward(x)) == top)
    let (far, near) = group.partition(across)
    for x in near:
        for y in far: bit[x, y] = top
    split(near)
    split(far)

split(Cut.to_list())
print("!")
for u in 0..n: print([bit[u, v] for v in u + 1..=n])
