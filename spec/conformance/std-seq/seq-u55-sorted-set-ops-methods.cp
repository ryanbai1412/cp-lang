// expect: ok, output "1 2 3 5 2 2 1 3"
// Acceptance for seq/U55 (union, intersection, difference of sorted lists), handbook/product/std/sequences-usecases.md: each is a method on the first list.
let a = [1, 2, 2, 3]
let b = [2, 2, 5]
print(a.sorted_union(b), a.sorted_intersect(b), a.sorted_diff(b))
