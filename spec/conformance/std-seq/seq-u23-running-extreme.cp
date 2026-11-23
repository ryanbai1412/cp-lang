// expect: ok, output "3 3 4"
// Acceptance for seq/U23 (running extremes), handbook/product/std/sequences-usecases.md.
let p = [3, 1, 4].scan(max)
print(p.prefix(1), p.prefix(2), p.all)
