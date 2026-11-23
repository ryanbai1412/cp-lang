// expect: ok, output "2 1"
// Acceptance for seq/U41 (count each distinct value), handbook/product/std/sequences-usecases.md.
let t = [3, 1, 3].tally()
print(t[3], t[1])
