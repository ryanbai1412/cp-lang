// CCC 2005 S5 Pinball Ranking: after each game, its rank is one more than
// the number of earlier games with a strictly higher score. Print the
// average rank to two decimals.
let t: 1..=1e5 = read()
let scores: [t]0..=1e9 = read()

var seen = SortedBag<Int>()                                         // WISH F13: a bag with rank queries
var ranks = 0
for s in scores:
    ranks += 1 + seen.count_above(s)
    seen.add(s)
print(ranks / F64(t), digits: 2)                                    // WISH F15: floats are still undecided
