// Syntax Batch #19: a list beside `+` takes the element type asked for,
// and an empty list in `print` takes no space.
type Z = Mod<998244353>
let rest: List<Z> = [Z(2), Z(3)]
let xs: List<Z> = [0] + rest
let ys: List<F64> = [1] + [2.5]
let zs: List<Z> = rest + [0, 1]
let ws: List<Z> = [1] + [2] + rest
fn pad(a: List<Z>) -> List<Z>:
    return [0] + a + [0]
let e: List<Int> = []
print(xs, ys, zs, ws, pad(rest))
print(1, e, 2)
print(e)
print(e, 5)
print(e, e)
print(5, e)
print("a", "", "b")
var q = [1, 2]
print(e, q, e, 7, end: "!\n")
