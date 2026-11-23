// expect: ok, output "3 2\n0 2 4"
// rule: 07.7.11
// `s.step(k)` takes every k-th element of a list as of a range.
let a = [3, 1, 2, 4]
print([x for x in a.step(2)])
print([i for i in (0..5).step(2)])
