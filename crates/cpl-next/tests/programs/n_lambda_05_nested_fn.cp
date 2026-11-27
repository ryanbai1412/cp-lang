// expect: ok, output "24 17 13 true 15 12"
// rule: R05.10.5, R06.2.1
// A nested `fn` is a fn value with captures; the fns of one run may call each other.
fn outer() -> Int:
    fn fact(n: Int) -> Int = if n == 0: 1 else: n * fact(n - 1)
    fact(4)
fn walk(n: Int, k: Int) -> (Int, Int):
    fn down(i: Int) -> Int = if i <= 0: 0 else: k + up(i - 1)
    fn up(i: Int) -> Int = if i <= 0: 0 else: 1 + down(i - 1)
    (down(n), up(n))
fn parity(n: Int) -> Bool:
    fn even(i: Int) -> Bool = if i == 0: true else: odd(i - 1)
    fn odd(i: Int) -> Bool = if i == 0: false else: even(i - 1)
    even(n)
fn twice(f: Int -> Int, x: Int) -> Int = f(f(x))
fn tally(xs: List<Int>) -> Int:
    var total = 0
    fn add(x: Int):
        total += x
    fn first_big(limit: Int) -> Int:
        for x in xs:
            if x > limit: return x
        return 0
    for x in xs: add(x)
    total + first_big(5) - first_big(100)
fn bump(x: Int) -> Int:
    fn inc(y: Int) -> Int = y + x
    twice(inc, 0)
let (a, b) = walk(5, 5)
print(outer(), a, b, parity(10), tally([1, 2, 6]), bump(6))
