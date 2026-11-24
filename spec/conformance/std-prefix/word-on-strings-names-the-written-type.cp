// expect: E0606
// `prefix(a)` is `Prefix(a)`: a list of strings has no `zero` to start the sums from, and the note names `Prefix.new` as written.
let xs = ["a", "b"]
let p = prefix(xs) // <- here
print(p.cut(1))
