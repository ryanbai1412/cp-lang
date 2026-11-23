// The flag is cleared again, so it says more than whether `last` is set.
let n: 1..=100 = read()
let a: [0..n]Int = read()
var open = false
var last = 0
for i in 0..n:
    if a[i] > 0:
        last = a[i]
        open = true
    else:
        open = false
print(yes(open), last)
