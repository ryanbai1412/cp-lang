// A sum of squares where the sum was meant, planted for `cpl stress --explain`.
let n: 1..=5 = read()
let a: [n]0..=9 = read()

var s = 0
for x in a: s += x * x
print(s)

@brute(main)
fn slow():
    var s = 0
    for x in a: s += x
    print(s)
