// expect: ok, output "5\n1 2 1 10 3 0 99"
// Only code after a statement that always leaves is dropped (bf5699d04): an `if` without `else`, a `break` in a loop and a match with one leaving arm all fall through.
enum E = A | B(k: Int)
fn f(c: Int) -> Int:
    if c > 0:
        return 1
    else:
        return 2
fn g(c: Int) -> Int:
    if c > 0:
        return 1
    c + 10
fn h(xs: List<Int>) -> Int:
    var s = 0
    for x in xs:
        if x < 0:
            break
        s += x
    s
fn m(e: E) -> Int:
    match e:
        A => return 0
        B(k) => print(k)
    99
let a = m(E.A)
let b = m(E.B(5))
print(f(1), f(0), g(1), g(0), h([1, 2, -1, 5]), a, b)
