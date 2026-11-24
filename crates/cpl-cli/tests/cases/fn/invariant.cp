// cp: 0.1 010af1d+dirty
// A property of a fn is an `assert` on its own result: the best pair sum doesn't depend on the order,
// and the first minus the last changes sign when the array is reversed.
let n: 1..=20 = read()
domain P = 1..=n
let a: [P]-100..=100 = read()

fn max_pair_sum(xs: [P]-100..=100) -> Int:
    var best = -1000
    for i in P:
        for j in P:
            if i < j: best = max(best, xs[i] + xs[j])
    best

fn first_minus_last(xs: [P]-100..=100) -> Int:
    xs[P.first!] - xs[P.last!]

let r: [P]-100..=100 = [P] i => a[P.nth(P.count - 1 - i.ord)]
assert max_pair_sum(r) == max_pair_sum(a)
assert first_minus_last(r) == -first_minus_last(a)
print(max_pair_sum(a))
