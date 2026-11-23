@!multitest
let (n, x): (1..=2e5, 1..=1e9) = read()
let w: [0..n]1..=1e9 = read()
let s = w.sorted()
var i = 0
var j = n - 1
var boats = 0
while i <= j:
    if i < j and s[i] + s[j] <= x: i += 1
    j -= 1
    boats += 1
print(boats)
