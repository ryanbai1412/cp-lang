// Fns pass values: a callee never changes what its caller holds unless
// the parameter says `mut`.
var calls = 0

fn gcd(a: Int, b: Int) -> Int:
    calls += 1
    if b == 0: a else: gcd(b, a % b)

fn fill(mut xs: List<Int>, n: Int):
    for i in 0..n:
        xs.push(i * i)

fn total(xs: List<Int>) -> Int:
    var t = 0
    for x in xs:
        t += x
    t

fn bumped(xs: List<Int>) -> List<Int>:
    var ys = xs.clone()
    for i in 0..ys.len:
        ys[i] += 1
    ys

fn swap(mut a: Int, mut b: Int):
    let t = a
    a = b
    b = t

fn fact(n: Int) -> Int:
    if n <= 1:
        return 1
    n * fact(n - 1)

fn is_even(n: Int) -> Bool:
    if n == 0: true else: is_odd(n - 1)

fn is_odd(n: Int) -> Bool:
    if n == 0: false else: is_even(n - 1)

fn classify(n: Int) -> Str:
    if n < 0:
        return "negative"
    if n == 0:
        return "zero"
    "positive"

// A parameter the fn assigns to is its own copy.
fn countdown(n: Int) -> Int:
    var steps = 0
    var left = n
    while left > 0:
        left -= 3
        steps += 1
    steps

var log: List<Int> = []

fn note(x: Int) -> Int:
    log.push(x)
    x

// The fn changes the global it was handed a value of: the value it was
// handed must not move.
fn grow_and_sum(xs: List<Int>) -> Int:
    log.push(99)
    total(xs)

print(gcd(48, 18), gcd(17, 5), calls)
var xs: List<Int> = []
fill(mut xs, 5)
print(xs, total(xs))
let ys = bumped(xs)
print(xs, ys)
var a = 1
var b = 2
swap(mut a, mut b)
print(a, b)
swap(mut xs[0], mut xs[4])
print(xs)
print(fact(10), fact(20), yes(is_even(10)), yes(is_odd(7)), classify(-3), classify(0), classify(8), countdown(10))
// Arguments run left to right.
print(note(1) + note(2) * note(3), log)
print(note(4) - note(5), min(note(6), note(7)), log)
print(grow_and_sum(log), log.len)
