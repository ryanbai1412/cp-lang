// Batch #12: a fn inside a fn takes `mut` parameters; the call works on the caller's variable.
let n: Int = read()
var total = 0
for round in 0..2:
    var par: [0..=n]Int = -1
    var seen: List<Int> = []
    fn touch(mut par: [0..=n]Int, x: Int) -> Int:
        par[x] = round
        x + 1
    fn note(mut xs: List<Int>, mut count: Int, x: Int):
        xs.push(x)
        count += x
        if x > 0: note(mut xs, mut count, x - 1)
    print(touch(mut par, 0), touch(mut par, n))
    note(mut seen, mut total, 2 + round)
    print(par, seen, total)
