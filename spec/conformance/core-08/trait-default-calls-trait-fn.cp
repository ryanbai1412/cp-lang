// expect: ok, output "assoc YES 7"
// rule: 08.1.1
// A trait default's body calls the trait's own fns unqualified; each impl's copy calls that impl's fns.
trait Op<T>:
    fn op(a: T, b: T) -> T
    fn twice(self, a: T) -> T = op(a, a)
    fn checks(self) -> List<(name: Str, holds: (List<T>, List<Int>) -> Bool)> = [
        (
            name: "assoc",
            holds: (s: List<T>, p: List<Int>) => op(op(s[p[0]], s[p[1]]), s[p[2]]) == op(
                s[p[0]],
                op(s[p[1]], s[p[2]]),
            ),
        )
    ]
struct S: x: Int
impl Op<Int> for S:
    fn op(a: Int, b: Int) -> Int = a + b
struct Mx
impl Op<Int> for Mx:
    fn op(a: Int, b: Int) -> Int = if a > b: a else: b
let l = S(0).checks()
print(l[0].name, yes(l[0].holds([1, 2, 3], [0, 1, 2])), S(0).twice(2) + Mx().twice(3))
