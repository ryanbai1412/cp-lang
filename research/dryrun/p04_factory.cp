// CSES 1620 Factory Machines: least time T with sum(T / k_i) >= t.
let (n, t): (1..=2e5, 1..=1e9) = read()
let k: [0..n]1..=1e9 = read()
fn enough(time: Int) -> Bool:
    var made = 0
    for x in k:
        made += time / x
        if made >= t: return true
    false
print(bsearch_first(1..=1e18, enough)!)
