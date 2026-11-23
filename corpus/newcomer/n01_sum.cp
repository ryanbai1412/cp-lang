let n: 1..=200000 = read()
let a: [0..n]Int = read()
var s = 0
for i in 0..n:  // expect: loop_reduction
    s = s + a[i]  // expect: compound_self_ref|loop_reduction
print(s)
