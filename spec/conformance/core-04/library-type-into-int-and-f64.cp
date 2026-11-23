// expect: ok, output "34 3.4000000000 <3,4> 7\n12 56"
// rule: 04.2.28
// `Int(x)` and `F64(x)` on a library type call its `impl Into<Int>` / `impl Into<F64>`; `Str(x)` is its `Show` (decision 390).
struct Pair(a: Int, b: Int)
impl Into<Int> for Pair:
    fn into(self) -> Int = a * 10 + b
impl Into<F64> for Pair:
    fn into(self) -> F64 = F64(a) + F64(b) / 10.0
impl Show for Pair:
    fn show(self) -> Str = "<{a},{b}>"
struct Sub<B>(bits: Int)
impl Into<Int> for Sub<B>:
    fn into(self) -> Int = bits
domain V = 0..3
let p = Pair(3, 4)
let s: Sub<V> = Sub(6)
print(Int(p), F64(p), Str(p), Int(s) + 1)
print([Int(q) for q in [Pair(1, 2), Pair(5, 6)]])
