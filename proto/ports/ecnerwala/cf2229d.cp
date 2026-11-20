// Codeforces 2229 D: binary search on the threshold v such that grouping
// consecutive (-1) runs leaves strictly more (+1) columns than (-1) runs.
@!multitest
let n: Int = read()
let a0: [0..n]Int = read()
let a1: [0..n]Int = read()

fn is_good(v: Int) -> Bool:
    var was_neg = false
    var tot = 0
    for i in 0..n:
        var s = -1
        if a0[i] >= v: s += 1
        if a1[i] >= v: s += 1
        if s == 0: continue
        if s == -1:
            if was_neg: continue
            was_neg = true
            tot -= 1
        else:
            was_neg = false
            tot += 1
    tot > 0

print(bsearch_last(1..=2 * n, v => is_good(v))!)
