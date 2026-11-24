// expect: E0203
// has: "s.to_upper()"
// The foreign name comes from `@aka(upper, ..)` on std's `to_upper`, not a compiler table.
let s = "ab"
print(s.upper()) // <- here
