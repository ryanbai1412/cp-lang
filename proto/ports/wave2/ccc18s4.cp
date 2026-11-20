// CCC 2018 S4 Balanced Trees: a tree of weight w splits into k >= 2 equal
// subtrees of weight w / k, rounded down. Count the trees of weight n, taking
// together every k that gives the same subtree weight.
let n: 1..=1e9 = read()

@memo
fn trees(w: 1..=1e9) -> Int:
    if w == 1: return 1
    var (total, k) = (0, 2)
    while k <= w:
        let last = w / (w / k)          // the largest k with the same subtree weight
        total += (last - k + 1) * trees(w / k)
        k = last + 1
    total
print(trees(n))
