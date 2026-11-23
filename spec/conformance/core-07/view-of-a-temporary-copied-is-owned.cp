// expect: ok, output "3"
// rule: 07.5.10
// Copying a view of a temporary with `List(...)` gives an owned value, so a `let` may keep it (R07.5.10).
let w = List([3, 1, 2].sorted()[0..2])
print(w.sum())
