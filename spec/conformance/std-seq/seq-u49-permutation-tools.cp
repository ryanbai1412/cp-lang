// expect: ok, output "YES 1 3 2\n6"
// Acceptance for seq/U49 (permutations: next, cycles, rank, parity), handbook/product/std/sequences-usecases.md.
var a = [1, 2, 3]
print(yes(a.next_permutation()), a)
print([1, 2, 3].permutations().count())
