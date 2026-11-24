// here: 4
// C++'s range `for (auto x : a)` (self-run mistakes): "expected `,` or `)` in the pattern, found a name".
let a = [3, 1, 2]
for (const auto &x : a) print(x)
