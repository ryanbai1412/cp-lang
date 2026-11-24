// A struct fn field that one capturing closure reaches is that closure's
// own type: a captured list is copied into it, so a holder returned past
// the list's scope keeps it, and a captured var is shared, so the field
// sees each later write.
struct Keep(f: Int -> Int)
struct Share(f: Int -> Int)

fn make(n: Int) -> Keep:
    let xs = [i * i for i in 0..n]
    Keep(i => xs[i] + n)

fn tally(k: Int) -> Int:
    var total = 0
    let add = Share(x => total + x * k)
    var s = 0
    for i in 0..4:
        total += i
        s += add.f(i)
    s

let p = make(6)
var held = [make(3), make(5)]
print(p.f(5))
print(held[0].f(2) + held[1].f(4))
print(tally(3))
