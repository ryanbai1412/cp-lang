// Codeforces 2222 B: split values by index parity, sort each group, drop the
// largest element plus up to (cnt - 1) additional positive elements, and sum.
@!multitest
let (n, m): (Int, Int) = read()
let a_in: [0..n]Int = read()
let x_in: [0..m]Int = read()

var even_v: List<Int> = []
var odd_v: List<Int> = []
for i in 0..n:
    if i % 2 == 0: even_v.push(a_in[i])
    else: odd_v.push(a_in[i])

var cnt0 = 0
var cnt1 = 0
for j in 0..m:
    if (x_in[j] - 1) % 2 == 0: cnt0 += 1
    else: cnt1 += 1

fn best_rem(vals: List<Int>, var cnt: Int) -> Int:
    var s = sorted(vals)
    if cnt > 0:
        cnt -= 1
        s.pop()
        while cnt > 0 and not s.is_empty and s.last! > 0:
            cnt -= 1
            s.pop()
    sum(s)

print(best_rem(even_v, cnt0) + best_rem(odd_v, cnt1))
