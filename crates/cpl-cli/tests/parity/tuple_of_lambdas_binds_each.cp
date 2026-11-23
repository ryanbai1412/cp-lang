var made = 0

fn shift(base: Int) -> Int:
    made += 1
    base

fn main_like(n: Int) -> Int:
    let at = shift(10)
    let (pre, suf) = ((i: Int) => at + i, (i: Int) => at + n + i)
    var t = 0
    for i in 0..n:
        t += pre(i) * suf(i)
    t

let (inc, twice) = ((x: Int) => x + 1, (x: Int) => 2 * x)
print(inc(twice(5)), twice(inc(5)))
print(main_like(3))
let pre = 100
let (pre2, keep) = ((i: Int) => pre + i, (i: Int) => i)
print(pre2(1), keep(7))
print(made)
