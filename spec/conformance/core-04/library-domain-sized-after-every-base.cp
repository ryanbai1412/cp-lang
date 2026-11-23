// expect: ok, output "9"
// input: "3\n"
// rule: 04.2.26
// R04.2.26: a `count` that reads two domains is sized once both are, so neither is read unset.
struct P<A, B>(i: Int)
impl Index for P<A, B>:
    let count = A.count * (12 / B.count)
    @get fn ord(self) -> Int = i
    fn nth(k: Int) -> P<A, B> = P(k)
let n: Int = read()
domain X = 0..n
domain Y = 0..n + 1
var a: [P<X, Y>]Int = 0
print(P<X, Y>.count)
