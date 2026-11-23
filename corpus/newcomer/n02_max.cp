let n: 1..=100 = read()
let a: [0..n]Int = read()
var best = -1000000000000000000  // expect: missing sentinel-instead-of-optional
for x in a:
    if x > best:  // expect: chmin_chmax_if
        best = x
print(best)
