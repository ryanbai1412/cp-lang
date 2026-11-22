type Z = Mod<998244353>
let n: Int = read()
let a: List<Z> = List.filled(n, 0)
var b: List<Z> = List.filled(n, 1)
b.push(5)
print(a, b)
fn zeros(k: Int) -> List<Z>:
    return List.filled(k, 0)
print(zeros(2))
var g: List<List<Z>> = List.filled(2, List.filled(2, 7))
print(g)
let f: List<F64> = List.filled(2, 1.0)
print(f)
struct S:
    xs: List<Z>
let s = S(List.filled(2, 3))
print(s.xs)
