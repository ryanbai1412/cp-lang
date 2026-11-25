// expect: ok, output "51"
// rule: decision 40
// Round 5 finding 24: a generic fn called with its caller's own local domain must use that domain's size, per call.
fn count<D>(a: [D]Int) -> Int:
    var c = 0
    for v in D: c += 1
    c
fn run(n: Int) -> Int:
    domain V = 0..n
    var a: [V]Int = 0
    if n > 1: count(a) * 10 + run(n - 4) else: count(a)
print(run(5))
