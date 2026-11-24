// expect: R0401
// `span(lo, hi)` runs std's body, which traps when `lo > hi` on both engines.
domain D = 0..4
let r = span(D(3).l, D(1).l) // <- here
print(r.len)
