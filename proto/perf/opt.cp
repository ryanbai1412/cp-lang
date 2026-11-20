// cp: 0.1 5df8551
let n: 1..=1e7 = read()
domain I = 0..n
var best: [I]?Int = none
var x = 12345
for i in I:
    x = (x * 1103515245 + 12345) % 2147483648
    if x % 3 != 0:
        best[i] = x
var s = 0
for i in I:
    s += best[i] ?? 0
print(s)
