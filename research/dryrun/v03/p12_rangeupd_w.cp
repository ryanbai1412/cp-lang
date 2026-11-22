// p12 with a hand-written lazy segment tree; a tag (mul, add) maps x to mul * x + add (row 10).
let (n, q): (1..=2e5, 1..=2e5) = read()
let t = read<[n]1..=1e6>()
var sum: [0..4 * n]Int = 0
var mul: [0..4 * n]Int = 1
var add: [0..4 * n]Int = 0
fn put(k: Int, len: Int, m: Int, a: Int):
    sum[k] = m * sum[k] + a * len
    mul[k] = m * mul[k]
    add[k] = m * add[k] + a
fn build(k: Int, l: Int, r: Int):
    if r - l == 1:
        sum[k] = t[l]
        return
    let mid = (l + r) / 2
    build(2 * k, l, mid)
    build(2 * k + 1, mid, r)
    sum[k] = sum[2 * k] + sum[2 * k + 1]
fn push(k: Int, l: Int, r: Int):
    let mid = (l + r) / 2
    put(2 * k, mid - l, mul[k], add[k])
    put(2 * k + 1, r - mid, mul[k], add[k])
    mul[k] = 1
    add[k] = 0
fn update(k: Int, l: Int, r: Int, ql: Int, qr: Int, m: Int, a: Int):
    if qr <= l or r <= ql: return
    if ql <= l and r <= qr:
        put(k, r - l, m, a)
        return
    push(k, l, r)
    let mid = (l + r) / 2
    update(2 * k, l, mid, ql, qr, m, a)
    update(2 * k + 1, mid, r, ql, qr, m, a)
    sum[k] = sum[2 * k] + sum[2 * k + 1]
fn query(k: Int, l: Int, r: Int, ql: Int, qr: Int) -> Int:
    if qr <= l or r <= ql: return 0
    if ql <= l and r <= qr: return sum[k]
    push(k, l, r)
    let mid = (l + r) / 2
    query(2 * k, l, mid, ql, qr) + query(2 * k + 1, mid, r, ql, qr)
build(1, 0, n)
for _ in 0..q:
    let kind = read<1..=3>()
    let (a, b) = read<(1..=n, 1..=n)>()
    if kind == 3:
        print(query(1, 0, n, a - 1, b))
    else:
        let x = read<1..=1e6>()
        update(1, 0, n, a - 1, b, if kind == 1: 1 else: 0, x)
