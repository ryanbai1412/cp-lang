// Runnable port (proto/cp.py lib-run) of dmopc21c4p4.cp
// original: corpus/submissions/4131060.cpp

const MOD = 1_000_000_007
type Z = Mod<MOD>

let n: Int = read()
let comb = Comb<Mod<MOD>>(n)
let step = Z(n - 1).inv()

var ans: Z = 0
var scale = step
for i in 2..=n:
    scale *= step
    ans += comb.choose(n, i) * comb.fact(i - 1) * scale
print(ans.v)
