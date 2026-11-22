// SPOJ AGGRCOW: t cases; put c cows into n stalls at distinct positions x_i to maximize
// the smallest gap. "Gap >= d is possible" is greedy-checkable and true -> false in d.
@!multitest
let n = read<2..=1e5>()
let c = read<2..=n>()
let xs = read<[n]0..=1e9>().sorted()
fn fits(d: Int) -> Bool:
    var placed = 1
    var last = xs.first!
    for x in xs.drop(1) if x - last >= d:
        placed += 1
        last = x
    placed >= c
print((1..=1e9).last_true(fits)!)
