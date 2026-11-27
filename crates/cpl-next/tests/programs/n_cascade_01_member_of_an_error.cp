// expect: E0203
// rule: diagnostics
// lacks: "windows"
// lacks: "slice:"
// One root cause gives one diagnostic (spec/diagnostics.md): a member, method, index or operator
// on an expression already diagnosed is that error again and says nothing more.
let xs = [1, 2, 3]
let w = xs.nosuch // <- here
print(w.windows(2).len, w[true], w + 1, w.y.z)
