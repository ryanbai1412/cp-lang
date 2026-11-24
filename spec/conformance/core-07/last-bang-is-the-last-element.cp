// expect: ok, output "4 1 4"
// rule: 07.2.17
// Keys never wrap: the last element is `a.last!` and the `k`-th from the end `a[a.len - k]` (decision 508).
let a = [3, 1, 4]
print(a.last!, a[a.len - 2], a[a.len - 1])
