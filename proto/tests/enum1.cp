// cp: 0.1 a3da8be+dirty
let n: 1..=10 = read()
domain P = 1..=n
enum Op:
    Add(i: P, x: -100..=100)
    Ask(i: P)
let q: 1..=10 = read()
let ops: [q]Op = read()
var a: [P]Int = 0
for op in ops:
    match op:
        Add(i, x) =>
            a[i] += x
        Ask(i) =>
            print(a[i])
