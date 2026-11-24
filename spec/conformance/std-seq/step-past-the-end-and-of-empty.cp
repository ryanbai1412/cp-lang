// expect: ok, output "1 1 0 3 6"
// rule: 07.7.11
// A stride past the end keeps only the first element, a stride of an empty list is empty, and `step(1)` is the whole list.
let a = [1, 2].step(5)
let z: List<Int> = []
let w = [4, 5, 6].step(1)
print(a.len, a.nth(0), z.step(2).len, w.len, w.nth(2))
