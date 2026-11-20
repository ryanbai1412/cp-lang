// Runnable port (proto/cp.py lib-run) of etsr23r4p2.cp
// original: corpus/submissions/5529698.cpp
use std.algebra.{Monoid, Action}
use std.lazy.{LazySegTree}
use std.mod.{Mod}

fn phi(x: Int) -> Int:
    var ans = x
    var rem = x
    var d = 2
    while d * d <= rem:
        if rem % d == 0:
            ans = ans / d * (d - 1)
            while rem % d == 0: rem /= d
        d += 1
    if rem > 1:
        ans = ans / rem * (rem - 1)
    ans

fn pow_capped(base: Int, exp: Int, m: Int) -> (Bool, Int):
    if m == 1: return (true, 0)
    var over = false
    var b_over = false
    var ans = 1
    var b = base
    var e = exp
    while e > 0:
        if (e & 1) == 1:
            if b_over or ans * b >= m: over = true
            ans = (ans * b) % m
        if b * b >= m: b_over = true
        b = (b * b) % m
        e = e >> 1
    (over, ans)

let (n, q, p, base): (Int, Int, Int, Int) = read()
type Z = Mod<p>
domain I = 1..=n
let a: [I]Int = read()

var phis: List<Int> = []
var cur_m = p
while true:
    phis.push(cur_m)
    if cur_m == 1: break
    cur_m = phi(cur_m)

let slots = phis.len + 1
domain Slot = 0..slots

fn tower(steps: Int, x: Int, depth: Int) -> (Bool, Int):
    if steps == 0:
        return (x >= phis[depth], x % phis[depth])
    let next_d = min(phis.len - 1, depth + 1)
    let sub = tower(steps - 1, x, next_d)
    let exp = if sub.0: sub.1 + phis[next_d] else: sub.1
    let res = pow_capped(base, exp, phis[depth])
    (sub.0 or res.0, res.1)

fn add_vec(x: List<Z>, y: List<Z>) -> List<Z>:
    [x[i] + y[i] for i in 0..slots]

fn shift_vec(k: Int, x: List<Z>, _len: Int) -> List<Z>:
    if k == 0: return x
    [x[min(i + k, slots - 1)] for i in 0..slots]

let node_m = Monoid<List<Z>>(List.filled(slots, Z(0)), add_vec)
let shift_act = Action<List<Z>, Int>(0, (a, b) => min(a + b, slots - 1), shift_vec, over: node_m)

var st = LazySegTree<I, List<Z>, Int>(
    shift_act,
    [I] i => [Z(tower(s.to_idx, a[i], 0).1) for s in Slot],
)

for _ in 0..q:
    let (ty, l, r): (Int, Int, Int) = read()
    if ty == 0:
        st.apply(I(l)..=I(r), 1)
    else:
        print(st.fold(I(l)..=I(r))[0])
