@!multitest
let n: 1..=100 = read()
let a: [0..n]Int = read()
var ok = true
for i in 0..n - 1:  // expect: search_loop
    if a[i] > a[i + 1]:
        ok = false
if ok:  // expect: bool_ternary_print
    print("YES")
else:
    print("NO")
