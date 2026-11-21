@!interactive
const V = 1_000_000
const NUM_P = 150

fn repack(a: List<Int>, src_w: Int, dst_w: Int, dst_n: Int) -> List<Int>:
    var bits = List<Int>()
    for x in a:
        for b in 0..src_w: bits.push((x >> b) & 1)
    var out = [0 for _ in 0..dst_n]
    for (pos, bit_v) in bits.items:
        let idx = pos / dst_w
        if idx < dst_n: out[idx] |= bit_v << (pos % dst_w)
    out

let sv = Sieve(V)
let max_p = sv.primes[NUM_P - 1]
var smooth = [1]
var zi = 0
while zi < smooth.len:
    let x = smooth[zi]
    zi += 1
    for p in sv.primes:
        if p > max_p or x * p > V: break
        smooth.push(x * p)
        if p == sv.spf[x]: break

var smooth_idx = [-1 for _ in 0..=V]
for i in 0..(1 << 18): smooth_idx[smooth[i]] = i

var p_pows = List<Int>()
for i in 0..NUM_P:
    let p = sv.primes[i]
    var q = p
    while q <= V / p: q *= p
    p_pows.push(q)

fn encode(a: List<Int>, smooth: List<Int>, p_pows: List<Int>) -> List<Int>:
    var b = [smooth[v] for v in repack(a, 20, 18, ceil_div(a.len * 20, 18))]
    for q in p_pows: b.push(q)
    b

let run: Str = read()
let t: Int = read()
for _ in 0..t:
    if run == "first":
        let n: Int = read()
        let a = [read::<Int>() for _ in 0..n]
        let b = encode(a, smooth, p_pows)
        print(b.len)
        print(b)
    else:
        let (n, k_tot): (Int, Int) = read()
        let k = k_tot - NUM_P
        var b = [0 for _ in 0..k]
        for i in 0..k:
            var val = 1
            for j in 0..NUM_P:
                print("?", i + 1, k + j + 1)
                let g: Int = read()
                if g == -1: return
                val *= g
            b[i] = smooth_idx[val]
        let a = repack(b, 18, 20, n)
        print("!", a)
