// expect: E0302
// rule: 04.2.28 08.5.2
// has: "impl Into<Int> for Pair"
// A struct with no `impl Into<Int>` has no `Int(x)`: there are no implicit conversions, and an `Into<F64>` doesn't stand in for it (decision 390).
struct Pair(a: Int, b: Int)
impl Into<F64> for Pair:
    fn into(self) -> F64 = F64(a)
print(Int(Pair(1, 2))) // <- here
