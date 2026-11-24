// expect: ok, output "0 1 2\n3\n0\n1\na b a\n10\n0"
// rule: 07.7.3
// `take(k)` on a generator stops pulling after `k` values, so an infinite one gives its first `k`,
// in a fold, a `for`, a generic generator and a closure alike (R07.1.3).
var pulled = 0
fn nat() -> yields Int:
    var i = 0
    while true:
        pulled += 1
        yield i
        i += 1
fn cycle<T>(xs: List<T>) -> yields T:
    while true:
        for x in xs: yield x
print(nat().take(3).collect())
print(pulled)
for x in nat().take(2): print(x)
print(cycle(["a", "b"]).take(3).collect())
let first = (n: Int) => nat().take(n).sum()
print(first(5))
print(nat().take(0).sum() + nat().take(-1).sum())
