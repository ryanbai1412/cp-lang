// expect: R0101
// rule: 07.2.17
// A computed negative key does not wrap to the end as in Python: it is out of range and traps (decision 508).
let a = [3, 1, 4]
let k = a.len - 4
print(a[k]) // <- here
