let lv = [0..=2, 0..3] (i, j) =>
    let miss = if j > 0: i + j else: i * 10
    miss + 1
print(lv[0, 0], lv[1, 0], lv[2, 2])

var xs = [12, 3, 21, 5]
xs.sort(by: x =>
    let d = x % 10
    (d, x)
)
print(xs)

fn apply(v: Int, f: fn(Int) -> Int) -> Int:
    f(v)

let r = apply(7, x =>
    var acc = 0
    for k in 1..=x:
        acc += k
    acc
)
print(r)

let cnt = [1, 2, 3, 4].count(x =>
    let sq = x * x
    sq > 4
)
print(cnt)
