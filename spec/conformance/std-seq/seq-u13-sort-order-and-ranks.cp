// expect: ok, output "1 2 0 2 0 1"
// Acceptance for seq/U13 (the order of a sort, and its inverse), handbook/product/std/sequences-usecases.md.
let w = [30, 10, 20]
let ord = w.keys.sorted(by: i => w[i])
print(ord, ord.inverse())
