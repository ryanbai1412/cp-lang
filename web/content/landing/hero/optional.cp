let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
var best: ?Int = None
for x in a:
    if x % 2 == 0: best max= x
print(best ?? -1)
