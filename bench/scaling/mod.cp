// let: P = 1000000007; p = 2; e = \log n
// fixed: ks_len, m, omega, p_i, e_i
use std.mod.{Barrett, PrimePowerComb, ModComb}
type Z = Mod<1_000_000_007>
let n: Int = read()
var lg = 0
while 1 << lg < n: lg += 1
var acc = 0
let br = Barrett(998_244_353)
let mc = ModComb(360)
for i in 1..8 * n:
    acc ^= Z(i).v // timed: mod.Mod.new
    acc ^= Z.from_int(-i).v // timed: mod.Mod.from_int
    acc ^= Z(i).inv().v // timed: mod.Mod.inv
    acc ^= (Z(i).try_inv() ?? Z(0)).v // timed: mod.Mod.try_inv
    acc ^= Barrett(i + 1).m // timed: mod.Barrett.new
    acc ^= br.mul(i % br.m, acc % br.m) // timed: mod.Barrett.mul
    acc ^= br.reduce(U64(i) * U64(i)) // timed: mod.Barrett.reduce
for i in 1..n:
    acc ^= mc.choose(i, i / 3) // timed: mod.ModComb.choose
    acc ^= ModComb(360).choose(9, 4) // timed: mod.ModComb.new
for i in 1..n / 4:
    acc ^= (Z(i).sqrt() ?? Z(0)).v // timed: mod.Mod.sqrt
let w = [Z(i) for i in 1..n + 1]
for _ in 0..16:
    acc ^= Z.inv_all(w)[0].v // timed: mod.Mod.inv_all
    let c = Comb<Z>(n) // timed: mod.Comb.new
    acc ^= c.fact(n - 1).v
let c = Comb<Z>(2 * n + 2)
for i in 0..8 * n:
    let j = i % n
    acc ^= c.fact(j).v // timed: mod.Comb.fact
    acc ^= c.inv_fact(j).v // timed: mod.Comb.inv_fact
    acc ^= c.inv(j + 1).v // timed: mod.Comb.inv
    acc ^= c.choose(j, j / 2).v // timed: mod.Comb.choose
    acc ^= c.arrange(j, j / 3).v // timed: mod.Comb.arrange
    acc ^= c.multichoose(j, j / 2).v // timed: mod.Comb.multichoose
    acc ^= c.catalan(j).v // timed: mod.Comb.catalan
    acc ^= c.derange(j).v // timed: mod.Comb.derange
    acc ^= c.multinomial([j / 3, j / 5, 2]).v // timed: mod.Comb.multinomial
for _ in 0..24:
    acc ^= PrimePowerComb(2, lg).choose(5, 2) // timed: mod.PrimePowerComb.new
let ppc = PrimePowerComb(2, lg)
for i in 0..2 * n:
    acc ^= ppc.choose(i % n, i % 7) // timed: mod.PrimePowerComb.choose
for i in 1..n / 4:
    acc ^= (Mod<1009>(11).log(Mod<1009>(i % 1008 + 1)) ?? 0) // timed: mod.Mod.log
print(acc)
