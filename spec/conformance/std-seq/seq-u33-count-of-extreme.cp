// expect: ok, output "3 2"
// Acceptance for seq/U33 (how many times the extreme occurs), handbook/product/std/sequences-usecases.md.
let top = [3, 1, 3].max_count()!
print(top.val, top.count)
