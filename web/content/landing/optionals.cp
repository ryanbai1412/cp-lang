let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]Int = read()

var best: ?Int = None
for x in a:
    if x % 2 == 0:
        best max= x
print(best ?? -1)
