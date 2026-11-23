// 1933E Turtle vs. Rabbit Race: Optimal Trainings
// in: t; per test n, a_1..a_n, q, then q lines "l u". out: per test, the q chosen r on one line
fn gain(s: Int, u: Int) -> Int = s * u - s * (s - 1) / 2

@!multitest
let n: Int = read()
let a: [n]Int = read()
let pre = a.scan(+)
let q: Int = read()
var out: List<Int> = []
repeat q:
    let l, u: Int = read()
    let s = l - 1
    let r = (s..n).last_true(e => pre[e + 1] - pre[s] <= u) ?? s
    let r2 = min(r + 1, n - 1)
    let best = if gain(pre[r2 + 1] - pre[s], u) > gain(pre[r + 1] - pre[s], u): r2 else: r
    out.push(best + 1)
print(out)
