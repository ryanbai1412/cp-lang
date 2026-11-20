const MOD = 1000000007
type Z = Mod<MOD>

@!multitest
let (n, l, q): (Int, Int, Int) = read()
let a: List<Int> = [read::<Int>() - 1 - i for i in 0..n]
var ops: List<(Int, Int)> = []
for _ in 0..q:
    let (i0, x0): (Int, Int) = read()
    let i = i0 - 1
    ops.push((x0 - 1 - i, i))
ops.sort()

let comb = Comb<MOD>()
let fact_q = comb.fact(q)
let inv_val: [0..=q]Z = [0..=q] i => if i == 0: 0 else: Z(i).inv()

var out: [0..n]Z = 0
for i in 0..n:
    var num_left = 0
    var num_right = 0
    for z in 0..q:
        if ops[z].1 <= i: num_right += 1
    var ans: Z = 0
    for z in 0..=q:
        let lo = if z == 0: 0 else: ops[z - 1].0
        let hi = if z == q: l else: ops[z].0
        if hi > lo:
            if num_left + num_right == 0:
                ans += Z(min(max(a[i], lo), hi) - lo)
            else:
                ans += Z(hi - lo) * Z(num_right) * inv_val[num_left + num_right]
        if z < q:
            if ops[z].1 <= i: num_right -= 1
            if ops[z].1 >= i: num_left += 1
    out[i] = (ans + Z(i + 1)) * fact_q
print(out)
