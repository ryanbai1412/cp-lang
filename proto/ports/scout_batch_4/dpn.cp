// Educational DP N, Slimes: merge adjacent slimes until one is left. Merging
// two costs their combined size. Minimise the total cost.
// original: corpus/submissions/4131779.cpp
let n: 1..=400 = read()
domain I = 0..n
type Cut = I.bounds
let a: [I]1..=1e9 = read()
let size = prefix(a)

// cost(l, r): cheapest way to merge the slimes between cuts l and r into one.
// The last merge joins the two sides of a cut strictly between them.
@memo
fn cost(l: Cut, r: Cut) -> Int:
    if (l..r).len < 2: return 0
    min(cost(l, m.l) + cost(m.l, r) for m in l..r if m.l != l)! + size.sum(l..r)
print(cost(Cut.first, Cut.last))
