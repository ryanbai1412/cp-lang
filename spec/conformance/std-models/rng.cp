// expect: ok, output "ok"
// input: "1 300\n"
// model: Rng
use std.rand.{Rng}

domain V = 0..6

// The reference splitmix64 step, written out from Vigna's constants.
fn mix(mut s: U64) -> U64:
    s = s + U64(-7046029254386353131)
    var z = s
    z = (z ^ (z >> 30)) * U64(-4658895280553007687)
    z = (z ^ (z >> 27)) * U64(-7723592293110705685)
    z ^ (z >> 31)

fn in_range(r: U64, lo: Int, hi: Int) -> Int =
    Int(I128(lo) + I128(r) % (I128(hi) - I128(lo) + I128(1)))

fn sorted_eq(a: List<Int>, b: List<Int>) -> Bool = a.sorted() == b.sorted()

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var s = U64(seed)
var twin = Rng(seed)
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 7)
    if op == 0:
        let got = twin.next()
        let want = mix(mut s)
        if got != want: bad = "op {k}: next {got} want {want}"
    elif op == 1:
        let lo = rng.int(-1000000000000000000, 1000000000000000000)
        let hi = lo + rng.int(0, 1000000000000000000)
        let got = twin.int(lo, hi)
        let want = in_range(mix(mut s), lo, hi)
        if got != want or got < lo or got > hi: bad = "op {k}: int({lo}, {hi}) = {got} want {want}"
    elif op == 2:
        let lo = rng.int(-5, 5)
        let hi = lo + rng.int(0, 3)
        let got = twin.int(lo, hi)
        let want = in_range(mix(mut s), lo, hi)
        if got != want or got < lo or got > hi: bad = "op {k}: int({lo}, {hi}) = {got} want {want}"
    elif op == 3:
        let got = twin.float()
        let want = F64(Int(mix(mut s) >> 11)) / 9007199254740992.0
        if got != want or got < 0.0 or got >= 1.0: bad = "op {k}: float {got} want {want}"
    elif op == 4:
        let p = F64(rng.int(0, 4)) / 4.0
        let got = twin.chance(p)
        let want = F64(Int(mix(mut s) >> 11)) / 9007199254740992.0 < p
        if got != want: bad = "op {k}: chance({p}) = {yes(got)} want {yes(want)}"
    elif op == 5:
        let xs = [rng.int(-9, 9) for _ in 0..rng.int(0, 5)]
        let got = twin.pick(xs)
        let want: ?Int = if xs.len == 0: None else: Some(xs[in_range(mix(mut s), 0, xs.len - 1)])
        if got != want: bad = "op {k}: pick({xs}) = {got ?? 99} want {want ?? 99}"
    elif op == 6:
        let xs = [rng.int(-9, 9) for _ in 0..rng.int(0, 7)]
        var got = xs.clone()
        twin.shuffle(mut got)
        var want = xs.clone()
        for i in (1..want.len).rev: want.swap_idx(i, in_range(mix(mut s), 0, i))
        if got != want or not sorted_eq(got, xs): bad = "op {k}: shuffle({xs}) = {got} want {want}"
    else:
        let xs = [rng.int(-9, 9) for _ in 0..rng.int(0, 7)]
        let m = rng.int(0, xs.len)
        let got = twin.sample(m, from: xs)
        var pos = (0..xs.len).collect()
        for i in 0..m: pos.swap_idx(i, in_range(mix(mut s), i, xs.len - 1))
        let want = [xs[pos[i]] for i in 0..m]
        if got != want: bad = "op {k}: sample({m}, {xs}) = {got} want {want}"
        let p = twin.perm<V>()
        var q: List<Int> = (0..6).collect()
        for i in (1..6).rev: q.swap_idx(i, in_range(mix(mut s), 0, i))
        if bad == "" and [p[V(i)].ord for i in 0..6] != q: bad = "op {k}: perm {p} want {q}"
print(if bad == "": "ok" else: bad)
