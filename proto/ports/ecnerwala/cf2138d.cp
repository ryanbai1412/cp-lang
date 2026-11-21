type Z = Mod<1_000_000_007>
let comb = Comb<Z.MOD>()

@!multitest
let (n, l, q): (Int, Int, Int) = read()
let raw_a: [0..n]Int = read()
let a: [0..n]Int = [0..n] i => raw_a[i] - 1 - i
let raw_ops: [0..q](Int, Int) = read()
let ops = [(val: x - i, pos: i - 1) for (i, x) in raw_ops].sorted()

let fact_q = comb.fact(q)
let inv: [0..=q]Z = [0..=q] i => if i == 0: 0 else: Z(i).inv

var out: [0..n]Z = 0
for i in 0..n:
    let ai = a[i]
    var num_left = 0
    var num_right = count(pos <= i for (_, pos) in ops)
    var ans: Z = 0
    var lo = 0

    fn sweep(hi: Int):
        if hi > lo:
            if num_left + num_right == 0:
                ans += Z(min(max(ai, lo), hi) - lo)
            else:
                ans += Z(hi - lo) * Z(num_right) * inv[num_left + num_right]
        lo = hi

    for (hi, pos) in ops:
        sweep(hi)
        if pos <= i: num_right -= 1
        if pos >= i: num_left += 1
    sweep(l)

    out[i] = (ans + Z(i + 1)) * fact_q

print(out)
