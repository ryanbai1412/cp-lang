let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let top = a.max()
var second: ?Int = None
for x in a:
    if x < top: second max= x
print(second ?? -1)
