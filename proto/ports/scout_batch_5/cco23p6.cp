// CCO 2023 Triangle Collection: have[i] sticks of length i. A triangle is
// two equal sticks of length a and a third shorter than 2a. After every
// change of one count, print the most triangles that can be built at once.
// original: corpus/submissions/5618725.cpp
use std.segtree.{SegTree}                                           // WISH F1

let n: 1..=2e5 = read()
let q: 1..=2e5 = read()
domain Len = 1..=n
var have: [Len]0..=1e9 = read()
let changes: [q](len: Len, by: -1e9..=1e9) = read()

// Pair up equal sticks. A pair of length a takes any odd stick out up to
// length 2a - 1, so file the pair at that length: scanning lengths upwards,
// an odd stick is used by a pair filed at or after it.
struct Run:
    made, singles, pairs: Int               // singles wait for a pair further right; pairs wait for a single further left
    op join(a, b) -> Run:                                           // WISH K13: the monoid lives with its record
        let k = min(a.singles, b.pairs)
        Run(a.made + b.made + k, a.singles + b.singles - k, a.pairs + b.pairs - k)
    const empty = Run(0, 0, 0)

fn reach(i: Len) -> Len = Len.clamp(2 * i.label - 1)                // WISH K16
fn pairs_of(i: Len) -> Int = have[i] / 2
let filed = Counter(reach(i) for i in Len, weight: pairs_of)        // WISH K13: a derived table that follows `have`
let runs = SegTree.of(Len, i => Run(0, have[i] % 2, 0).join(Run(0, 0, filed[i])))   // WISH F7

for (i, by) in changes:
    have[i] += by                           // `filed` and `runs` refresh the two slots that read have[i]
    let spare = pairs_of.sum() - runs.all.made
    print(runs.all.made + spare * 2 / 3)    // three spare pairs make two triangles
