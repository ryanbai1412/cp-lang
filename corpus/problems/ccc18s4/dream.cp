// CCC 2018 S4 Balanced Trees: a tree of weight w > 1 has k >= 2 identical children of weight w / k.
// Count the trees of weight n. Group k by the value of n / k: O(n^(3/4)) states.
@memo
fn trees(w: Int) -> Int =
    if w == 1: 1
    else: w.quotients(from: 2).sum(of: q => q.count * trees(q.value))
print(trees(read()))
