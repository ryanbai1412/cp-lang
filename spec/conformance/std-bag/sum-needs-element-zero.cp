// expect: E0606
// rule: R08.2.2
// has: "does not implement `Zero`"
// `SortedBag.sum` states `where K: Add + Zero`, the bounds its body uses, so a bag whose
// element type has no `Zero` has no `sum`.
let b = SortedBag.of(["a", "b"])
print(b.sum()) // <- here
