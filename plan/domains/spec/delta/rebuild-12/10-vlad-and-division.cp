// 1926D Vlad and Division
// in: t; per test n, a_1..a_n (0 <= a_i < 2^31). out: minimum number of groups, per test
const full = (1 << 31) - 1

@!multitest
let n: Int = read()
let a: [n]Int = read()
var open: Counter<Int, Int> = Counter(0)
var groups = 0
for x in a:
    if open[x ^ full] > 0:
        open.add(x ^ full, -1)
    else:
        open.add(x, 1)
        groups += 1
print(groups)
