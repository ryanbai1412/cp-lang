// YAC 6 P5 (today/ runnable subset) — original: corpus/submissions/5149823.cpp

let n: 1..=200_000 = read()
var xs: List<(Int, Int)> = []
var ys: List<(Int, Int)> = []
for i in 0..n:
    let x: -1_000_000_000..=1_000_000_000 = read()
    let y: -1_000_000_000..=1_000_000_000 = read()
    let v: 0..=3 = read()
    var bx = 1
    if v == 0 or v == 1:
        bx = 0
    var by = 1
    if v == 0 or v == 3:
        by = 0
    xs.push((x, bx))
    ys.push((y, by))

var pre: [0..=n]Int = 0

// cost of gathering sorted positions l..r at their median
fn cost(l: Int, r: Int) -> Int:
    let h = (r - l) / 2
    (pre[r] - pre[r - h]) - (pre[l + h] - pre[l])

fn solve(mut arr: List<(Int, Int)>) -> Int:
    sort(mut arr)
    var cadd = 0
    for i in 0..n:
        pre[i + 1] = pre[i] + arr[i].0
        if arr[i].1 == 0:
            cadd += 1
    var best = cost(0, n)
    for i in 0..=n:
        let cur = cost(0, i) + cost(i, n) + cadd
        if cur < best:
            best = cur
        if i < n:
            if arr[i].1 == 0:
                cadd -= 1
            else:
                cadd += 1
    best

print(solve(mut xs) + solve(mut ys))
