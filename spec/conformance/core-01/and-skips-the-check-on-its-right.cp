// expect: ok, output "NO YES NO"
// rule: 01.5.7
// The right side of `and`, `or` and `??` runs only when it is needed, and so does the range check
// of a cut inside it: `k > 0 and Town.cuts(k - 1) == ..` never builds the cut `-1`.
domain Town = 1..=4
let k = 0
let first = Town.cuts.first!
print(yes(k > 0 and Town.cuts(k - 1) == first), yes(k == 0 or Town.cuts(k - 1) == first), yes((Some(first) ?? Town.cuts(k - 1)) != first))
