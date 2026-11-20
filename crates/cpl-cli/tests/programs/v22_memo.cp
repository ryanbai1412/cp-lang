// A nested @memo fn: its table belongs to one call of the fn around it.
let n: 1..=60 = read()
fn count(step: 1..=3) -> Int:
    @memo
    fn ways(left: 0..=n) -> Int:
        if left == 0: return 1
        sum(ways(left - s) for s in 1..=step if s <= left)
    ways(n)
for step in 1..=3: print(count(step))

domain I = 1..=n
@memo
fn fib(i: I, twice: Bool) -> Int:
    let base = if i.label <= 2: 1 else: fib(i.prev!, false) + fib(i.prev!.prev!, false)
    if twice: 2 * base else: base
print(fib(I.last, false), fib(I.last, true))

@memo
fn collatz(x: Int) -> Int:
    if x == 1: 0 elif x % 2 == 0: 1 + collatz(x / 2) else: 1 + collatz(3 * x + 1)
print(max(collatz(x) for x in 1..=n)!)

@memo
fn word(k: Int, tag: Str) -> Str:
    if k == 0: tag else: word(k - 1, tag) + word(k - 1, tag)
print(word(3, "ab").len)
