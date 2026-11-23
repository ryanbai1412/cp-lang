let n: 1..=200000 = read()
let a: [0..n]Int = read()
var p: List<(Int, Int)> = []
for i in 0..n:  // expect: push_loop
    p.push((a[i], i))
let s = p.sorted()  // expect: missing pair-sort-for-argsort
for (v, i) in s:
    print(i + 1, end: " ")
print()
