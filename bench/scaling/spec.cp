// n: 512
use std.spec.{decrease, increase, convexity_break, concavity_break, unimodality_break, monge_break, argmins, dnc_break, knuth_break, differences, newton, poly_degree, poly_at, berlekamp_massey, recurrence_at, guess_at}
type Z = Mod<998_244_353>
let n: Int = read()
let up = [i for i in 0..n]
let down = [n - i for i in 0..n]
let convex = [(i - n / 2) * (i - n / 2) for i in 0..n]
let concave = [0 - (i - n / 2) * (i - n / 2) for i in 0..n]
let cubic = [i * i * i % 1_000_003 for i in 0..n]
var fib: List<Z> = []
var seed = 1
for _ in 0..n:
    seed = seed * 48271 % 2147483647
    fib.push(Z(seed))
let half = List(fib[0..n / 2])
let monge: (Int, Int) -> Int = (i, j) => (i - j) * (i - j) // within: spec.monge_break
let arg: (Int, Int) -> Int = (i, j) => (i - j) * (i - j) // within: spec.argmins
let dnc: (Int, Int) -> Int = (j, i) => (i - j) * (i - j) // within: spec.dnc_break
let opt: (Int, Int) -> Int = (i, j) => (i + j) / 2 // within: spec.knuth_break
var s = 0
for _ in 0..12000:
    s += decrease(up) ?? 0 // timed: spec.decrease
    s += increase(down) ?? 0 // timed: spec.increase
    s += convexity_break(convex) ?? 0 // timed: spec.convexity_break
    s += concavity_break(concave) ?? 0 // timed: spec.concavity_break
    s += unimodality_break(up) ?? 0 // timed: spec.unimodality_break
    s += differences(cubic).len // timed: spec.differences
for _ in 0..16:
    if monge_break(monge, n, n) == None: s += 1 // timed: spec.monge_break
    s += argmins(arg, n, n).len // timed: spec.argmins
    if dnc_break(dnc, n) == None: s += 1 // timed: spec.dnc_break
    if knuth_break(opt, n) == None: s += 1 // timed: spec.knuth_break
    s += newton(cubic).len // timed: spec.newton
    s += poly_degree(cubic) ?? 0 // timed: spec.poly_degree
    s += poly_at(cubic, 3 * n) // timed: spec.poly_at [let: len = n]
    s += berlekamp_massey(fib).len // timed: spec.berlekamp_massey
    s += recurrence_at(half, half, 1 << 40).v // timed: spec.recurrence_at [fixed: n]
for _ in 0..16:
    s += (guess_at(fib, 1 << 40) ?? Z(0)).v // timed: spec.guess_at [fixed: n]
print(s)
