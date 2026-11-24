// here: 4
// C++'s `v.push_back({a, b})` on a list of pairs (self-run mistakes): read as a set.
var v: List<(Int, Int)> = []
v.push_back({1, 2})
print(v)
