// expect: E0309
// rule: 01.8.5
let a = [1, 2, 3, 4, 5]
print(sum(a[0..5 by 2])) // <- here
