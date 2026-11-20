// IOI '16 P1 - Detecting Molecules (DMOJ: ioi16p1)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// n molecules with weights w, and a detector range [l, u] that is at least
// as wide as the spread of the weights: u - l >= max(w) - min(w). Return the
// indices of any subset whose total weight is in [l, u], or an empty list.
//
// Sort by weight. If any subset works, a window of the sorted order works:
// grow the window on the right while the sum is below l, drop the lightest
// on the left while it is above u. Dropping moves the sum by at most
// max - min, so it cannot jump over [l, u].
@grader
fn find_subset(l: Int, u: Int, w: [I]Int) -> List<I>:
    domain I
    let order = sorted(I, by: i => w[i])
    var (lo, total) = (0, 0)
    for (hi, i) in order.enumerate:
        total += w[i]
        while total > u:
            total -= w[order[lo]]
            lo += 1
        if total >= l: return order[lo..=hi]
    []

@checker(find_subset)
fn valid(l: Int, u: Int, w: [I]Int, got: List<I>) -> Bool:
    if got.is_empty: return not any(l <= sum(w[i] for i in s) <= u for s in subsets(I))
    distinct(got) and l <= sum(w[i] for i in got) <= u
