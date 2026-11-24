// CCC 2018 S4 Balanced Trees: a tree of weight w > 1 has k >= 2 identical children of weight w / k.
// Count the trees of weight n. Group k by the value of n / k: O(n^(3/4)) states.
@memo
fn trees(w: Int) -> Int =
    if w == 1: 1
    else: sum((b.hi - b.lo) * trees(b.q) for b in floor_quotients(w, from: 2))
print(trees(read<1..=1e9>()))
