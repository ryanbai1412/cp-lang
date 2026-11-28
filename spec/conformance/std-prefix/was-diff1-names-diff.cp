// expect: E0201
// rule: 12.13.1
// The old name comes from `@was(Diff1, Diff2)` on std's `Diff`, not a compiler table.
var d = Diff1<Int>(3) // <- here
