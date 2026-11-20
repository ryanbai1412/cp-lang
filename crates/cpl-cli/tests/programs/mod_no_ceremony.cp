// Syntax batch 1 on `Mod<P>`: literals, compound assignment through an
// index, `Z.P`, and consts as generic arguments.
type Z = Mod<998244353>
var x: Z = 1
x += 2
x *= 3
x -= 1
print(x)
if x != 0: print("nz")
if x == 8: print("eight")
let n = 3
domain I = 0..n
var a: [I]Z = 1
var b: [I]Z = 2
for i in I: a[i] *= b[i]
for i in I: a[i] += 5
print(a)
fn f(k: Z) -> Z = k * 2 + 1
print(f(4))
var xs: List<Z> = [1, 2, 3]
xs.push(4)
print(xs)
print(Z.P, Mod<7>.P)
const P1 = 998244353
const P2 = 1000000007
fn two<const P>(x: Int) -> Mod<P> = Mod<P>(x) * 2
print(two::<P1>(P1 - 1))
print(two::<P2>(5))
type Y = Mod<P2>
let y: Y = 3
print(y ** 2)
