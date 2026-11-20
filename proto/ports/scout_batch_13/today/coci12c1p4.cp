// COCI 2012/13 #1 Ljubomora, as it builds today: only `ceil_div` is missing (F1).
// original: corpus/submissions/5245322.cpp
let kids: 1..=1e9 = read()
let m: 1..=3e5 = read() where m <= kids
let marbles: [m]1..=1e9 = read() where sum(marbles) > kids   // the original divides by zero when the answer is 1

// with handfuls of at most x, colour c needs ceil(c / x) children
print((1..=1e9).first_where(x => sum((c + x - 1) / x for c in marbles) <= kids)!)
