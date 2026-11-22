// expect: ok, output "6"
// rule: 01.2.4
let xs = [1, 2, 3]
let s = xs.map(x => x)
    .sum()
print(s)
