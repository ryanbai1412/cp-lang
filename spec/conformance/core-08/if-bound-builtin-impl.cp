// expect: ok, output "3 -1 2"
// `if T: Tr` holds for a built-in type with `impl Tr for Int`, the same as for a struct; a type
// with no such impl takes the other branch.
trait Fast:
    fn go(a: List<Self>) -> ?Int
impl Fast for Int:
    fn go(a: List<Int>) -> ?Int = Some(a.len)
struct W(v: Int)
impl Fast for W:
    fn go(a: List<W>) -> ?Int = Some(a.len - 1)
fn f<T>(a: List<T>) -> Int:
    if T: Fast:
        if let c = T.go(a): return c
    -1
print(f([1, 2, 3]), f([true]), f([W(1), W(2), W(3)]))
