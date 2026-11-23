// Codeforces 2229 D, tuned: port.cp's test as one pass per threshold, as
// orig.cpp does, instead of building the sign list and its nonzero filter.
@!multitest
let n: 1..=200_000 = read()
let a0: [0..n]1..=2 * n = read()
let a1: [0..n]1..=2 * n = read()

fn is_good(v: Int) -> Bool:
    var total = 0
    var was_neg = false
    for (x, y) in zip(a0, a1):
        let s = Int(x >= v) + Int(y >= v) - 1
        if s == 1:
            total += 1
            was_neg = false
        elif s == -1 and not was_neg:
            total -= 1
            was_neg = true
    total > 0

print((1..=2 * n).last_true(is_good)!)
