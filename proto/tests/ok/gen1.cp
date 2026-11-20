// cp: 0.1 329fa7d+dirty
let q: 1..=5 = read()
let xs: [q]1..=1e9 = read()

fn prime_factors(x: 1..=1e9) -> yields Int:
    var r = x
    var p = 2
    while p * p <= r:
        if r % p == 0:
            yield p
            while r % p == 0:
                r /= p
        p += 1
    if r > 1:
        yield r

fn odd_factors(x: 1..=1e9) -> yields Int:
    for p in prime_factors(x):
        if p != 2:
            yield p

fn both(x: 1..=1e9) -> yields Int:
    yield from odd_factors(x)
    yield 0

fn first_big(x: 1..=1e9) -> Int:
    for p in prime_factors(x):
        if p > 10:
            return p
    -1

for x in xs:
    var line: List<Int> = []
    for p in both(x):
        line.push(p)
    var firsttwo = 0
    for p in prime_factors(x):
        if firsttwo == 2:
            break
        firsttwo += 1
    print(line.len, first_big(x), firsttwo, sum(p for p in prime_factors(x)))
