// COCI 2012/13 #1 Ljubomora, as it builds today: only `ceil_div` is missing (F1).
// original: corpus/submissions/5245322.cpp
let kids: 1..=1e9 = read()
let m: 1..=3e5 = read()
assert m <= kids
let marbles: [m]1..=1e9 = read()
assert marbles.sum() > kids  // the original divides by zero when the answer is 1

// with handfuls of at most x, colour c needs ceil(c / x) children
print(bsearch_first(1..=1e9, x => sum(ceil_div(c, x) for c in marbles) <= kids)!)
