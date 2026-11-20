// original: 4143748 (coci21c2p5) -- first draft, written from docs/one-page-cheat-sheet.md only; does not build on proto/cpc
const LOG = 18
let n: Int = read()
let iv: [0..n](Int, Int) = read()
var all: List<Int> = []
for (l, r) in iv:
    all.push(l)
    all.push(r)
let (C, id, _) = compress(all)
var st = LazySegTree<C, Int>(AddMax, [C] c => 0)
fn add(i: Int, d: Int):
    if i < n: st.apply(id[2 * i]..=id[2 * i + 1], d)
var lift: [0..LOG, 0..=n]Int = n
var j = 0
add(0, 1)
for i in 0..n:
    while j < n and st.fold(C.first..=C.last) <= 1:
        j += 1
        add(j, 1)
    lift[0, i] = j
    add(i, -1)
for b in 1..LOG:
    for i in 0..n: lift[b, i] = lift[b - 1, lift[b - 1, i]]
let q: Int = read()
for _ in 0..q:
    let (l0, r0): (Int, Int) = read()
    var l = l0 - 1
    let r = r0 - 1
    var ans = 0
    for b in (0..LOG).rev:
        if lift[b, l] <= r:
            ans |= 1 << b
            l = lift[b, l]
    print(ans + 1)
