// expect: E0204
// The old name comes from `@was(Prefix1, Prefix2, PrefixN)` on std's `Prefix`.
fn f(p: Prefix2<Int, Int, Int>) -> Int = 0 // <- here
