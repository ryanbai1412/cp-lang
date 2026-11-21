// Codeforces 2222 B: split values by index parity, sort each group, drop the
// largest element plus up to (cnt - 1) additional positive elements, and sum.
@!multitest
let (n, m): (Int, Int) = read()
let a: [0..n]Int = read()
let xs: [0..m]Int = read()

let cnt0 = count(x % 2 == 1 for x in xs)
fn best_rem(vals: List<Int>, var cnt: Int) -> Int:
    var s = sorted(vals)
    if cnt > 0:
        cnt -= 1
        s.pop()
        while let x = s.last, cnt > 0 and x > 0:
            cnt -= 1
            s.pop()
    sum(s)

print(best_rem([a[i] for i in 0..n by 2], cnt0) + best_rem([a[i] for i in 1..n by 2], m - cnt0))
