// CCC 2018 S4 Balanced Trees: a tree of weight w splits into k >= 2 equal
// subtrees of weight floor(w / k). Count them, grouping the k that share a
// quotient.
let n: 1..=1e9 = read()

@memo
fn trees(w: 1..=1e9) -> Int:
    if w == 1: return 1
    var total = 0
    var k = 2
    while k <= w:
        let q = w / k
        let last = w / q                // the largest k with the same quotient
        total += (last - k + 1) * trees(q)
        k = last + 1
    total
print(trees(n))
