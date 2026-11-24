// An `assert` that fails inside a fn stops the run on the vm with its
// message, after what was printed before it.
fn check(k: Int, limit: Int) -> Int:
    assert k * k < limit, "{k} squared is {k * k}, not under {limit}"
    return k * k

let n: Int = read()
var total = 0
for k in 1..=n:
    total += check(k, 50)
    print(k, total)
