// `sum` adds any struct that has a `zero` and a `+`, and `Z.MOD` is the
// modulus of a `Mod`.
struct V2(x: Int, y: Int):
    let zero = V2(0, 0)
impl Add for V2:
    fn add(a, b) = V2(a.x + b.x, a.y + b.y)
type Z = Mod<998244353>
let vs = [V2(1, 2), V2(3, 4), V2(5, 6)]
let total = sum(vs)
print(total.x, total.y)
let t2 = sum(V2(i, i * i) for i in 0..4)
print(t2.x, t2.y)
let zs: List<Z> = [Z.MOD - 1, 2, 3]
print(sum(zs), Z.MOD, Z.P)
