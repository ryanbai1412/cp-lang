// expect: ok, output "0 1 2 3 3"
// Toward seq/U39 (a scan of a scan), handbook/product/std/sequences-usecases.md: a table over `D.cuts` walks its items, one per cut.
domain D = 0..3
let m: [D.cuts]Int = [D.cuts] c => c.ord
print([i.ord for (i, x) in m.items], m.max()!)
