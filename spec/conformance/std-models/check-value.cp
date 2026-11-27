// expect: R0102
// model: check_value
// Against brute force: an `Int` converts to a cut of `V` exactly when it is from 0 through
// `V.count` (R04.2.17). Every value the brute force accepts converts and reads back as itself;
// the first one past the ends traps.
domain V = 3..8
var ok = true
for k in 0..=V.count:
    ok = ok and Int(V.cuts(k)) == k
print(yes(ok))
print(Int(V.cuts(V.count + 1)))  // <- here
