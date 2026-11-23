let n: 1..=100 = read()
let a: [0..n]Int = read()
var mn = a[0]
var mx = a[0]
for i in 1..n:
    if a[i] < mn:  // expect: chmin_chmax_if
        mn = a[i]
    if a[i] > mx:  // expect: chmin_chmax_if
        mx = a[i]
print(mx - mn)
