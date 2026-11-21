// Nothing reads `trail`, `log` or `extra`, so the C++ does not store them,
// but what filling them does still happens: each `noisy` call prints and
// counts, in the same order as the interpreter.
let n: 1..=10 = read()
domain V = 0..n
var calls = 0
fn noisy(x: Int) -> Int:
    calls += 1
    print("made", x)
    x * 2

struct Run:
    total: Int
    trail: [V]Int
    log: List<Int>
    extra: Int

fn go() -> Run:
    var trail: [V]Int = 0
    var log = List<Int>()
    var total = 0
    for v in V:
        trail[v] = total
        log.push(noisy(total))
        total += 1
    Run(total, trail, log, noisy(100))

let r = go()
print(r.total, calls)
