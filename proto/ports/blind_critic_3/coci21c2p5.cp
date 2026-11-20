// original: 4143748 (coci21c2p5)
const LOG = 18
let n: Int = read()
let iv: [0..n](Int, Int) = read()
var all: List<Int> = []
for (l, r) in iv:
    all.push(l)
    all.push(r)
let (C, coord) = compress(all)
var st = AddMax<C>()
fn add(i: Int, d: Int):
    if i < n: st.add(coord.first_ge(iv[i].0)!, coord.first_ge(iv[i].1)!, d)
var lift: [0..LOG, 0..=n]Int = n
var j = 0
add(0, 1)
for i in 0..n:
    while j < n and st.max() <= 1:
        j += 1
        add(j, 1)
    lift[0, i] = j
    add(i, -1)
for b in 1..LOG:
    for i in 0..n: lift[b, i] = lift[b - 1, lift[b - 1, i]]
let q: Int = read()
for _ in 0..q:
    let l0: Int = read()
    let r0: Int = read()
    var l = l0 - 1
    let r = r0 - 1
    var ans = 0
    for b in (0..LOG).rev:
        if lift[b, l] <= r:
            ans |= 1 << b
            l = lift[b, l]
    print(ans + 1)
