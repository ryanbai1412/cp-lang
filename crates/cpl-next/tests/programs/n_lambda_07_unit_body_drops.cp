// expect: ok, output "4\n2 1"
// Where `()` is wanted, a lambda's or a one-line `fn`'s body is a statement and its value is
// dropped, as the same call last in a block is: `(mut d, e) => d.unite(e.0, e.1)` (R01.8.1,
// R06.1.1).
fn flag(mut x: Int) -> Bool:
    x += 1
    true
fn g(mut x: Int) = flag(mut x)
var a = 1
g(mut a)
let h: (mut Int) -> () = (mut s) => flag(mut s)
h(mut a)
let k: (mut Int) -> () = (mut s) => s += 1
k(mut a)
print(a)
fn each(xs: List<Int>, f: Int -> ()):
    for x in xs:
        f(x)
var seen: List<Int> = []
each([2, 1], x => seen.push(x))
print(seen[0], seen[1])
