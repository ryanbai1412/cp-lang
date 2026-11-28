// expect: ok, output "5 YES 6 4 3"
// Every number takes an `Int` through `FromInt.from_int`, the built-in ones too: `T.from_int(n)` for `T = Int` is `n` (decision 687).
struct W<T>(c: List<T>):
    fn one(v: Int) -> W<T> where T: FromInt = W([T.from_int(v)])
fn two<T: FromInt + Add>(v: Int, x: T) -> T = T.from_int(v) + x
print(W<Int>.one(5).c[0], yes(W<F64>.one(2).c[0] > 1.5), two(5, 1), Int.from_int(4), U64.from_int(3))
