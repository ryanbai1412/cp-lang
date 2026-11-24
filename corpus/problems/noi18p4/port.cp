// Source: NOI 2018 Day 2 P1 (Dragon Slayer), Luogu P4774 (not on DMOJ)
// limits: 2000 ms, 512 MB
// Problem: kill dragons 1..n in order (n, m <= 1e5, T <= 5). Dragon i (life a <= 1e12, regrows
//   p) dies iff a - x atk is <= 0 and a multiple of p, where atk is the strongest held sword
//   not above a (else the weakest), then swapped for the dragon's reward (atk <= 1e6). Print
//   the least hit count x that kills every dragon, or -1. lcm(p) <= 1e12.
// Solution: the sword per dragon is fixed: simulate with a SortedBag (predecessor, else
//   minimum). Dragon i needs atk x = a (mod p), one linear congruence via ext_gcd, and
//   x >= ceil(a / atk). Merge the congruences with general CRT (mul_mod against overflow),
//   then lift the residue to the least value >= the largest bound. O((n + m) log).
// original: corpus/submissions/6517796.cpp
use std.math.{ext_gcd, crt, mul_mod}

/// The solutions of a x = b (mod m) as one congruence x = r (mod m'), or none.
fn solve_linear(a: Int, b: Int, m: Int) -> ?(Int, Int):
    let (a, b) = (a % m, b % m)
    let g = gcd(a, m)
    if b % g != 0: return None
    let step = m / g
    (mul_mod(b / g, ext_gcd(a / g, step).x % step, step), step)

@multitest
let (n, m): (1..=1e5, 1..=1e5) = read()
domain Dragon = 0..n
domain Sword = 0..m
let life: [Dragon]1..=1e12 = read()
let regrow: [Dragon]1..=1e12 = read()
let reward: [Dragon]1..=1e6 = read()
var swords = SortedBag.of(read<[Sword]1..=1e6>().collect())

var at_least = 0
var all: ?(Int, Int) = (0, 1)
for d in Dragon:
    let atk = swords.range(..=life[d]).last ?? swords.first!
    swords.remove(atk)
    swords.add(reward[d])
    at_least max= ceil_div(life[d], atk)
    if let (r0, m0) = all and let (r1, m1) = solve_linear(atk, life[d], regrow[d]):
        all = crt(r0, m0, r1, m1)
    else:
        all = None
if let (r, md) = all:
    print(r + md * max(0, ceil_div(at_least - r, md)))
else:
    print(-1)
