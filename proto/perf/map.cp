// cp: 0.1 cc468f2+dirty
let n: 1..=1e7 = read()
domain I = 0..n
var last: Map<I, Int> = []
var x = 12345
for i in I:
    x = (x * 1103515245 + 12345) % 2147483648
    last[I.at(x % n)] = i.ord
var s = 0
for (k, v) in last:
    s += k.ord ^ v
print(last.len, s)
