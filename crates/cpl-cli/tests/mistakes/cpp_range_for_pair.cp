// here: 4
// C++'s `for (auto [u, v] : e)` (self-run mistakes): a bare pattern error.
let e = [(1, 2), (3, 4)]
for (auto [u, v] : e) print(u + v)
