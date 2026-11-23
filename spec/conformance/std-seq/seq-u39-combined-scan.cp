// expect: ok, output "0 2 4 2"
// Acceptance for seq/U39 (a scan of a scan), handbook/product/std/sequences-usecases.md.
let a = [1, -2, 3]
let b = [2, 2, -5]
let p = a.scan(+)
let q = b.scan(+)
print([0..=a.len] c => max(p.prefix(c), q.prefix(c)))
