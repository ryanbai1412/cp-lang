// expect: E0606
// rule: 12.4.6
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0: yield i

evens(10).sort() // <- here
